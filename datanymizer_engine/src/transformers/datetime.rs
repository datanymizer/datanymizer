use crate::transformer::{TransformContext, TransformResult, TransformResultHelper, Transformer};
use chrono::prelude::*;
use chrono::DateTime;
use fake::{faker::chrono::raw::*, locales::EN, Fake};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Eq, PartialOrd, Hash, Clone, Debug)]
pub struct FromValue(String);
#[derive(Serialize, Deserialize, PartialEq, Eq, PartialOrd, Hash, Clone, Debug)]
pub struct ToValue(String);
#[derive(Serialize, Deserialize, PartialEq, Eq, PartialOrd, Hash, Clone, Debug)]
pub struct Format(String);

const BOUNDS_FORMAT: &str = "%Y-%m-%dT%H:%M:%S%.f%:z";

/// Generates random dates.
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
#[derive(Serialize, Deserialize, PartialEq, Eq, Hash, Clone, Debug, Default)]
pub struct RandomDateTimeTransformer {
    #[serde(default)]
    pub from: FromValue,
    #[serde(default)]
    pub to: ToValue,
    #[serde(default)]
    pub format: Format,
}

impl Default for FromValue {
    fn default() -> Self {
        Self(Utc.ymd(1970, 1, 1).and_hms_micro(0, 0, 0, 0).to_rfc3339())
    }
}

impl Default for ToValue {
    fn default() -> Self {
        Self(Utc.ymd(9999, 1, 1).and_hms_micro(0, 0, 0, 0).to_rfc3339())
    }
}

impl Default for Format {
    fn default() -> Self {
        Self(String::from(BOUNDS_FORMAT))
    }
}

impl Transformer for RandomDateTimeTransformer {
    fn transform(
        &self,
        _field_name: &str,
        _field_value: &str,
        _ctx: &Option<TransformContext>,
    ) -> TransformResult {
        let from_dt = DateTime::parse_from_str(&self.from.0, BOUNDS_FORMAT)?.with_timezone(&Utc);
        let to_dt = DateTime::parse_from_str(&self.to.0, BOUNDS_FORMAT)?.with_timezone(&Utc);
        let between: chrono::DateTime<Utc> = DateTimeBetween(EN, from_dt, to_dt).fake();
        let res: String = between.format(&self.format.0).to_string();

        TransformResult::present(res)
    }
}

#[cfg(test)]
mod test {
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
                          to: 2010-10-31T00:00:00+00:00
                       "#;
        let result = transformed_value(cfg);
        assert!(result.starts_with("2010-10-"));
    }

    #[test]
    fn custom_format() {
        let cfg = r#"
                          format: "%m"
                          from: 2000-12-01T00:00:00+00:00
                          to: 2000-12-31T00:00:00+00:00
                       "#;
        let result = transformed_value(cfg);
        assert_eq!(result, "12");
    }
}
