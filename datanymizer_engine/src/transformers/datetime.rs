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
/// or custome date ranges:
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
/// Also, you can specify datetime format.
/// Be careful with it...
///
/// ```yaml
/// #...
/// rules:
///   birth_date:
///     datetime:
///       from: 1990-01-01T00:00:00+00:00
///       to: 2010-12-31T00:00:00+00:00
///       format: %Y-%m-%dT%H:%M:%S%.f%:z
/// ```
#[derive(Serialize, Deserialize, PartialEq, Eq, Hash, Clone, Debug)]
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
        Self(String::from("%Y-%m-%dT%H:%M:%S%.f%:z"))
    }
}

impl Default for RandomDateTimeTransformer {
    fn default() -> Self {
        Self {
            from: FromValue::default(),
            to: ToValue::default(),
            format: Format::default(),
        }
    }
}

impl Transformer for RandomDateTimeTransformer {
    fn transform(
        &self,
        _field_name: &str,
        _field_value: &str,
        _ctx: &Option<TransformContext>,
    ) -> TransformResult {
        let from_dt = DateTime::parse_from_str(&self.from.0, &self.format.0)?.with_timezone(&Utc);
        let to_dt = DateTime::parse_from_str(&self.to.0, &self.format.0)?.with_timezone(&Utc);
        let between: chrono::DateTime<Utc> = DateTimeBetween(EN, from_dt, to_dt).fake();
        let res: String = between.format(&self.format.0).to_string();

        TransformResult::present(res)
    }
}
