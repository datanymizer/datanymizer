use serde::{de::Error as DeError, Deserialize, Deserializer};
use serde_json::Value as JsonValue;
use std::{
    error::Error,
    fmt::{self, Display, Formatter},
};

#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct Assert {
    /// Assertion name shown in validation errors and warnings.
    pub name: String,
    /// SQL query executed against the source database.
    pub sql: String,
    /// Expected result of the SQL query.
    pub expect: AssertExpectation,
    /// Optional extra context appended to failures.
    pub message: Option<String>,
    #[serde(default)]
    /// Whether a failed assertion should stop dumping or only warn.
    pub severity: AssertSeverity,
}

/// Supported expectation kinds for SQL assertions.
#[derive(Debug, Clone, PartialEq)]
pub enum AssertExpectation {
    /// The query must return zero rows.
    NoRows,
    /// The query must return at least one row.
    RowsExist,
    /// The query must return a single scalar that satisfies all configured checks.
    Scalar(Box<ScalarExpectations>),
}

impl<'de> Deserialize<'de> for AssertExpectation {
    /// Parses either the short row-based form (`no_rows`, `rows_exist`) or
    /// the object form with scalar comparison operators.
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum RawExpectation {
            Keyword(String),
            Scalar(Box<ScalarExpectations>),
        }

        match RawExpectation::deserialize(deserializer)? {
            RawExpectation::Keyword(keyword) => match keyword.as_str() {
                "no_rows" => Ok(Self::NoRows),
                "rows_exist" => Ok(Self::RowsExist),
                _ => Err(D::Error::custom(format!(
                    "unknown assert expectation '{}'",
                    keyword
                ))),
            },
            RawExpectation::Scalar(expectations) => {
                if expectations.is_empty() {
                    Err(D::Error::custom(
                        "scalar assert expectation must contain at least one condition",
                    ))
                } else {
                    Ok(Self::Scalar(expectations))
                }
            }
        }
    }
}

#[derive(Debug, Deserialize, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct ScalarExpectations {
    /// The scalar value must be equal to this value.
    pub eq: Option<JsonValue>,
    /// The scalar value must differ from this value.
    pub not_eq: Option<JsonValue>,
    /// The scalar value must be greater than this number.
    pub gt: Option<JsonValue>,
    /// The scalar value must be greater than or equal to this number.
    pub gte: Option<JsonValue>,
    /// The scalar value must be less than this number.
    pub lt: Option<JsonValue>,
    /// The scalar value must be less than or equal to this number.
    pub lte: Option<JsonValue>,
}

impl ScalarExpectations {
    /// Returns `true` when the object form contains no comparison operators.
    fn is_empty(&self) -> bool {
        self.eq.is_none()
            && self.not_eq.is_none()
            && self.gt.is_none()
            && self.gte.is_none()
            && self.lt.is_none()
            && self.lte.is_none()
    }
}

#[derive(Debug, Deserialize, Clone, Copy, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum AssertSeverity {
    /// Stop the dump when the assertion fails.
    #[default]
    Error,
    /// Report the assertion failure and continue.
    Warn,
}

/// Resolved assertion scope used during execution and error reporting.
#[derive(Debug, Clone, Copy)]
pub enum AssertScope<'a> {
    Global(&'a Assert),
    Table(TableAssert<'a>),
}

/// Table-local assertion together with the table it belongs to.
#[derive(Debug, Clone, Copy)]
pub struct TableAssert<'a> {
    /// Table that owns the assertion.
    pub table: &'a super::Table,
    /// Assertion configuration defined for the table.
    pub assert: &'a Assert,
}

/// Rich assertion failure used by the runner and surfaced to the user.
#[derive(Debug, Clone, PartialEq)]
pub struct AssertError {
    /// Assertion name.
    pub name: Box<str>,
    /// Scope name (`global` or table name).
    pub scope: Box<str>,
    /// SQL text that produced the failure.
    pub sql: Box<str>,
    /// Human-readable expected condition.
    pub expected: Box<str>,
    /// Optional actual value or execution detail.
    pub actual: Option<Box<str>>,
    /// Optional user-provided message from config.
    pub message: Option<Box<str>>,
}

impl AssertScope<'_> {
    /// Returns the underlying assertion regardless of scope.
    pub fn assert(&self) -> &Assert {
        match self {
            Self::Global(assert) => assert,
            Self::Table(table_assert) => table_assert.assert,
        }
    }

    /// Returns a human-readable scope label for diagnostics.
    pub fn scope_name(&self) -> &str {
        match self {
            Self::Global(_) => "global",
            Self::Table(table_assert) => table_assert.table.name.as_str(),
        }
    }

    /// Validates the `no_rows` expectation from a row-existence check.
    pub fn expect_no_rows(self, has_rows: bool) -> Result<(), AssertError> {
        if has_rows {
            Err(self.error("expected no rows", Some("query returned rows")))
        } else {
            Ok(())
        }
    }

    /// Validates the `rows_exist` expectation from a row-existence check.
    pub fn expect_rows_exist(self, has_rows: bool) -> Result<(), AssertError> {
        if has_rows {
            Ok(())
        } else {
            Err(self.error("expected rows to exist", Some("query returned no rows")))
        }
    }

    /// Validates all scalar comparisons against a single scalar result.
    pub fn expect_scalar(self, actual: &JsonValue) -> Result<(), AssertError> {
        let expectations = match &self.assert().expect {
            AssertExpectation::Scalar(expectations) => expectations,
            _ => return Ok(()),
        };

        if let Some(eq) = &expectations.eq {
            if actual != eq {
                return Err(self.error(format!("expected {}", eq), Some(format!("got {}", actual))));
            }
        }

        if let Some(not_eq) = &expectations.not_eq {
            if actual == not_eq {
                return Err(self.error(
                    format!("expected value not equal to {}", not_eq),
                    Some(format!("got {}", actual)),
                ));
            }
        }

        if let Some(gt) = &expectations.gt {
            self.expect_numeric_comparison(actual, gt, ">", |a, b| a > b)?;
        }

        if let Some(gte) = &expectations.gte {
            self.expect_numeric_comparison(actual, gte, ">=", |a, b| a >= b)?;
        }

        if let Some(lt) = &expectations.lt {
            self.expect_numeric_comparison(actual, lt, "<", |a, b| a < b)?;
        }

        if let Some(lte) = &expectations.lte {
            self.expect_numeric_comparison(actual, lte, "<=", |a, b| a <= b)?;
        }

        Ok(())
    }

    /// Builds an error for scalar expectations when the query returned no rows.
    pub fn missing_scalar_value(self) -> AssertError {
        self.error("expected one scalar row", Some("query returned no rows"))
    }

    /// Builds a structured assertion error for the current scope.
    fn error<S1, S2>(self, expected: S1, actual: Option<S2>) -> AssertError
    where
        S1: Into<String>,
        S2: Into<String>,
    {
        let assert = self.assert();

        AssertError {
            name: assert.name.clone().into_boxed_str(),
            scope: self.scope_name().to_string().into_boxed_str(),
            sql: assert.sql.trim().to_string().into_boxed_str(),
            expected: expected.into().into_boxed_str(),
            actual: actual.map(Into::into).map(String::into_boxed_str),
            message: assert.message.clone().map(String::into_boxed_str),
        }
    }

    /// Validates a numeric comparison operator against a scalar result.
    fn expect_numeric_comparison<F>(
        self,
        actual: &JsonValue,
        expected: &JsonValue,
        op: &str,
        compare: F,
    ) -> Result<(), AssertError>
    where
        F: FnOnce(f64, f64) -> bool,
    {
        let actual_num = actual.as_f64().ok_or_else(|| {
            self.error(
                format!(
                    "expected numeric result for comparison '{} {}'",
                    op, expected
                ),
                Some(format!("got non-numeric scalar {}", actual)),
            )
        })?;
        let expected_num = expected.as_f64().ok_or_else(|| {
            self.error(
                format!(
                    "expected numeric comparison value for '{} {}'",
                    op, expected
                ),
                Some(format!("got non-numeric expectation {}", expected)),
            )
        })?;

        if compare(actual_num, expected_num) {
            Ok(())
        } else {
            Err(self.error(
                format!("expected {} {}", op, expected),
                Some(format!("got {}", actual)),
            ))
        }
    }
}

impl Display for AssertError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "assert '{}' failed in scope '{}': {}",
            self.name, self.scope, self.expected
        )?;

        if let Some(actual) = &self.actual {
            write!(formatter, "; {}", actual)?;
        }

        if let Some(message) = &self.message {
            write!(formatter, "; message: {}", message)?;
        }

        write!(formatter, "; sql: {}", self.sql)
    }
}

impl Error for AssertError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_expect_keyword() {
        let assert: Assert = serde_yaml::from_str(
            r#"
name: no_nulls
sql: select 1
expect: no_rows
"#,
        )
        .unwrap();

        assert_eq!(assert.expect, AssertExpectation::NoRows);
        assert_eq!(assert.severity, AssertSeverity::Error);
    }

    #[test]
    fn parse_expect_rows_exist() {
        let assert: Assert = serde_yaml::from_str(
            r#"
name: users_exist
sql: select 1 from users limit 1
expect: rows_exist
"#,
        )
        .unwrap();

        assert_eq!(assert.expect, AssertExpectation::RowsExist);
    }

    #[test]
    fn parse_scalar_expectations() {
        let assert: Assert = serde_yaml::from_str(
            r#"
name: users_count
sql: select count(*) from users
expect:
  gt: 0
  lt: 10
severity: warn
"#,
        )
        .unwrap();

        assert_eq!(
            assert.expect,
            AssertExpectation::Scalar(Box::new(ScalarExpectations {
                eq: None,
                not_eq: None,
                gt: Some(0.into()),
                gte: None,
                lt: Some(10.into()),
                lte: None,
            }))
        );
        assert_eq!(assert.severity, AssertSeverity::Warn);
    }

    #[test]
    fn reject_empty_scalar_expectations() {
        let error = serde_yaml::from_str::<Assert>(
            r#"
name: users_count
sql: select count(*) from users
expect: {}
"#,
        )
        .unwrap_err()
        .to_string();

        assert!(error.contains("at least one condition"));
    }

    #[test]
    fn expect_no_rows_failure_contains_context() {
        let assert = Assert {
            name: "users_email_unique".into(),
            sql: "select email from users".into(),
            expect: AssertExpectation::NoRows,
            message: Some("emails must be unique".into()),
            severity: AssertSeverity::Error,
        };

        let error = AssertScope::Global(&assert)
            .expect_no_rows(true)
            .unwrap_err();

        assert!(error.to_string().contains("users_email_unique"));
        assert!(error.to_string().contains("global"));
        assert!(error.to_string().contains("emails must be unique"));
        assert!(error.to_string().contains("select email from users"));
    }

    #[test]
    fn expect_rows_exist_failure_contains_context() {
        let assert = Assert {
            name: "users_exist".into(),
            sql: "select 1 from users limit 1".into(),
            expect: AssertExpectation::RowsExist,
            message: None,
            severity: AssertSeverity::Error,
        };

        let error = AssertScope::Global(&assert)
            .expect_rows_exist(false)
            .unwrap_err();

        assert!(error.to_string().contains("users_exist"));
        assert!(error.to_string().contains("expected rows to exist"));
    }

    #[test]
    fn expect_scalar_range_failure_contains_context() {
        let assert = Assert {
            name: "queue_size_in_range".into(),
            sql: "select count(*) from jobs".into(),
            expect: AssertExpectation::Scalar(Box::new(ScalarExpectations {
                eq: None,
                not_eq: None,
                gt: Some(5.into()),
                gte: None,
                lt: Some(10.into()),
                lte: None,
            })),
            message: None,
            severity: AssertSeverity::Error,
        };

        let error = AssertScope::Global(&assert)
            .expect_scalar(&JsonValue::from(3))
            .unwrap_err();

        assert!(error.to_string().contains("expected > 5"));
        assert!(error.to_string().contains("got 3"));
    }
}
