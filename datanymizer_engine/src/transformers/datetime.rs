use crate::{
    transformer::{TransformContext, TransformResult, TransformResultHelper, Transformer},
    utils::format_time::{CompileError, Compiled},
};
use rand::distributions::{Distribution, Uniform};
use serde::{Deserialize, Serialize};
use std::fmt::{Debug, Display, Formatter};
use std::hash::{Hash, Hasher};
use time::{format_description::well_known::Rfc3339, Duration, OffsetDateTime};

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
    parsed_from: OffsetDateTime,
    #[serde(skip)]
    parsed_to: OffsetDateTime,
    #[serde(skip)]
    compiled: Compiled,
}

impl RandomDateTimeTransformer {
    fn parse_date(s: &str) -> Result<OffsetDateTime, time::error::Parse> {
        OffsetDateTime::parse(s, &Rfc3339)
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

    fn try_from(c: Config) -> Result<Self, Self::Error> {
        let from = c.from;
        let to = c.to;
        let format = c.format;

        let parsed_from = Self::parse_date(from.as_str())?;
        let parsed_to = Self::parse_date(to.as_str())?;
        let compiled = Compiled::compile(format.as_str())?;

        Ok(Self {
            from,
            to,
            format,
            parsed_from,
            parsed_to,
            compiled,
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
        let duration = (self.parsed_to - self.parsed_from).whole_seconds();
        let mut rng = rand::thread_rng();
        let rnd_duration = Uniform::new_inclusive(0, duration).sample(&mut rng);

        let res = (self.parsed_from + Duration::seconds(rnd_duration))
            .format(&self.compiled.format_items())?;

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

#[derive(Debug)]
pub enum ParseError {
    Parse(time::error::Parse),
    Compile(CompileError),
}

impl Display for ParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Parse(e) => write!(f, "{e}"),
            Self::Compile(e) => write!(f, "{e}"),
        }
    }
}

impl std::error::Error for ParseError {}

impl From<time::error::Parse> for ParseError {
    fn from(err: time::error::Parse) -> Self {
        Self::Parse(err)
    }
}

impl From<CompileError> for ParseError {
    fn from(err: CompileError) -> Self {
        Self::Compile(err)
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
