use crate::transformer::{TransformContext, TransformResult, TransformResultHelper, Transformer};
use chrono::{DateTime, Duration, NaiveDateTime, ParseError, ParseResult};
use rand::distributions::{Distribution, Uniform};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use std::hash::{Hash, Hasher};

const BOUNDS_FORMAT: &str = "%Y-%m-%dT%H:%M:%S%.f%:z";

/// Generates random dates in the specified interval (granularity is a second).
///
/// # Example:
///
/// With default values:
///
/// ```yaml
/// #...
/// rules:
///   birth_date:
///     datetime: {}
/// ```
///
/// or custom date ranges:
///
/// ```yaml
/// #...
/// rules:
///   birth_date:
///     datetime:
///       from: 1990-01-01T00:00:00+00:00
///       to: 2010-12-31T00:00:00+00:00
/// ```
///
/// Also, you can specify datetime format for the output.
/// For the bounds (from/to) you should use the RFC 3339 format.
/// The default output format is also RFC 3339 (%Y-%m-%dT%H:%M:%S%.f%:z).
/// You don't need to change the format when using this transformer with datetime SQL fields.
/// More information about formatting is in the format.rs file.
///
/// ```yaml
/// #...
/// rules:
///   birth_date:
///     datetime:
///       from: 1990-01-01T00:00:00+00:00
///       to: 2010-12-31T00:00:00+00:00
///       format: %Y-%m-%d
/// ```
#[derive(Serialize, Deserialize, Eq, Clone, Debug)]
#[serde(try_from = "Config")]
pub struct RandomDateTimeTransformer {
    from: String,
    to: String,
    format: String,

    #[serde(skip)]
    parsed_from: NaiveDateTime,
    #[serde(skip)]
    parsed_to: NaiveDateTime,
}

impl RandomDateTimeTransformer {
    fn parse_date(s: &str) -> ParseResult<NaiveDateTime> {
        DateTime::parse_from_str(s, BOUNDS_FORMAT).map(|d| d.naive_utc())
    }
}

impl Hash for RandomDateTimeTransformer {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.from.hash(state);
        self.to.hash(state);
        self.format.hash(state);
    }
}

impl PartialEq for RandomDateTimeTransformer {
    fn eq(&self, other: &Self) -> bool {
        self.from == other.from && self.to == other.to && self.format == other.format
    }
}

impl TryFrom<Config> for RandomDateTimeTransformer {
    type Error = ParseError;

    fn try_from(c: Config) -> ParseResult<Self> {
        let from = c.from;
        let to = c.to;
        let format = c.format;

        let parsed_from = Self::parse_date(from.as_str())?;
        let parsed_to = Self::parse_date(to.as_str())?;

        Ok(Self {
            from,
            to,
            format,
            parsed_from,
            parsed_to,
        })
    }
}

impl Transformer for RandomDateTimeTransformer {
    fn transform(
        &self,
        _field_name: &str,
        _field_value: &str,
        _ctx: &Option<TransformContext>,
    ) -> TransformResult {
        let duration = (self.parsed_to - self.parsed_from).num_seconds();
        let mut rng = rand::thread_rng();
        let rnd_duration = Uniform::new_inclusive(0, duration).sample(&mut rng);

        let res = (self.parsed_from + Duration::seconds(rnd_duration)).format(&self.format);

        TransformResult::present(res)
    }
}

#[derive(Deserialize)]
#[serde(default)]
struct Config {
    from: String,
    to: String,
    format: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            from: String::from("1970-01-01T00:00:00+00:00"),
            to: String::from("9999-01-01T00:00:00+00:00"),
            format: String::from("%Y-%m-%dT%H:%M:%S%.f%:z"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn transformed_value(cfg: &str) -> String {
        let transformer: RandomDateTimeTransformer = serde_yaml::from_str(cfg).unwrap();
        transformer
            .transform("datetime", "", &None)
            .unwrap()
            .unwrap()
    }

    #[test]
    fn default_format() {
        let cfg = r#"
                          from: 2010-10-01T00:00:00+00:00
                          to: 2010-10-31T00:00:00.99+00:00
                       "#;
        let result = transformed_value(cfg);
        assert!(result.starts_with("2010-10-"));
    }

    #[test]
    fn custom_format_m() {
        let cfg = r#"
                          format: "%m"
                          from: 2000-12-01T00:00:00+00:00
                          to: 2000-12-31T00:00:00+00:00
                       "#;
        let result = transformed_value(cfg);
        assert_eq!(result, "12");
    }

    #[test]
    fn custom_format_h() {
        let cfg = r#"
                          format: "%H"
                          from: 2000-12-31T02:00:00+00:00
                          to: 2000-12-31T02:59:00+00:00
                       "#;
        let result = transformed_value(cfg);
        assert_eq!(result, "02");
    }

    #[test]
    fn custom_format_ymd() {
        let cfg = r#"
                          format: "%Y-%m-%d"
                          from: 2000-12-31T02:00:00+00:00
                          to: 2000-12-31T02:59:00+00:00
                       "#;
        let result = transformed_value(cfg);
        assert_eq!(result, "2000-12-31");
    }
}
