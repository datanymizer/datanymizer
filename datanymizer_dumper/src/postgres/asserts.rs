use super::query_wrapper::QueryWrapper;
use anyhow::{anyhow, Result};
use datanymizer_engine::{AssertExpectation, AssertScope, AssertSeverity, Settings};
use postgres::{Client, IsolationLevel};
use serde_json::Value as JsonValue;

/// Executes configured SQL assertions before dump generation starts.
pub fn run_asserts(
    settings: &Settings,
    client: &mut Client,
    isolation_level: Option<IsolationLevel>,
) -> Result<()> {
    let mut qw = QueryWrapper::with_isolation_level(client, isolation_level)?;
    run_asserts_with_query_wrapper(settings, &mut qw)
}

/// Runs all assertions with a shared query wrapper and collects warning-level failures.
fn run_asserts_with_query_wrapper(settings: &Settings, qw: &mut QueryWrapper<'_>) -> Result<()> {
    let mut warnings = Vec::new();

    for scope in settings.all_asserts() {
        if let Err(err) = run_assert(scope, qw) {
            match scope.assert().severity {
                AssertSeverity::Error => return Err(err),
                AssertSeverity::Warn => warnings.push(err.to_string()),
            }
        }
    }

    if warnings.is_empty() {
        Ok(())
    } else {
        eprintln!("Assertions warnings:");
        for warning in warnings {
            eprintln!("- {}", warning);
        }
        Ok(())
    }
}

/// Executes one assertion by choosing either a row-existence or scalar validation path.
fn run_assert(scope: AssertScope<'_>, qw: &mut QueryWrapper<'_>) -> Result<()> {
    let assert = scope.assert();

    match &assert.expect {
        AssertExpectation::NoRows => {
            let query = format!(
                "SELECT EXISTS(SELECT 1 FROM ({}) AS datanymizer_assert LIMIT 1)",
                assert.sql
            );
            let has_rows: bool = qw.query_one(query.as_str(), &[])?.get(0);
            scope.expect_no_rows(has_rows)?;
        }
        AssertExpectation::RowsExist => {
            let query = format!(
                "SELECT EXISTS(SELECT 1 FROM ({}) AS datanymizer_assert LIMIT 1)",
                assert.sql
            );
            let has_rows: bool = qw.query_one(query.as_str(), &[])?.get(0);
            scope.expect_rows_exist(has_rows)?;
        }
        AssertExpectation::Scalar(_) => {
            let query = format!(
                "SELECT to_jsonb(value) FROM ({}) AS datanymizer_assert(value)",
                assert.sql
            );
            let row = qw
                .query_opt(query.as_str(), &[])?
                .ok_or_else(|| anyhow!(scope.missing_scalar_value()))?;
            let actual: JsonValue = row.get(0);

            scope.expect_scalar(&actual)?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use datanymizer_engine::Assert;

    #[test]
    fn engine_assert_error_contains_context() {
        let assert = Assert {
            name: "users_email_unique".into(),
            sql: "select email from users".into(),
            expect: AssertExpectation::NoRows,
            message: Some("emails must be unique".into()),
            severity: AssertSeverity::Error,
        };

        let message = AssertScope::Global(&assert)
            .expect_no_rows(true)
            .unwrap_err()
            .to_string();

        assert!(message.contains("users_email_unique"));
        assert!(message.contains("global"));
        assert!(message.contains("emails must be unique"));
        assert!(message.contains("select email from users"));
    }
}
