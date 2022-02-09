use crate::transformer::{TransformContext, TransformResult, TransformResultHelper, Transformer};
use serde::{Deserialize, Serialize};
use std::hash::{Hash, Hasher};
use time::{
    format_description::{self, well_known::Rfc3339},
    OffsetDateTime,
};

const FORMAT_REPLACEMENTS: [(&str, &str); 22] = [
    ("%Y", "[year]"),
    ("%y", "[year repr:last_two]"),
    ("%m", "[month]"),
    ("%b", "[month repr:short]"),
    ("%B", "[month repr:long]"),
    ("%h", "[month repr:short]"),
    ("%d", "[day]"),
    ("%e", "[day padding:space]"),
    ("%a", "[weekday repr:short]"),
    ("%A", "[weekday]"),
    ("%w", "[weekday repr:sunday]"),
    ("%u", "[weekday repr:monday one_indexed:true]"),
    ("%U", "[week repr:sunday one_indexed:true]"),
    ("%W", "[week repr:monday one_indexed:true]"),
    ("%G", "[year base:iso_week]"),
    ("%g", "[year repr:last_two base:iso_week]"),
    ("%V", "[week]"),
    ("%j", "[ordinal]"),
    ("%D", "[month]/[day]/[year repr:last_two]"),
    ("%x", "[day].[month].[year repr:last_two]"),
    ("%F", "[year]-[month]-[day]"),
    ("%v", "[day padding:space]-[month repr:short]-[year]"),
];

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
#[derive(Serialize, Deserialize, Eq, Clone, Debug)]
#[serde(from = "Config")]
pub struct RandomDateTimeTransformer {
    from: String,
    to: String,
    format: String,

    #[serde(skip)]
    parsed_from: OffsetDateTime,
    #[serde(skip)]
    parsed_to: OffsetDateTime,
}

impl RandomDateTimeTransformer {
    fn parse_date(s: &str) -> Result<OffsetDateTime, time::error::Parse> {
        OffsetDateTime::parse(s, &Rfc3339)
    }

    fn prepare_format(s: &str) -> String {
        let mut s = s.to_string();
        for (from, to) in FORMAT_REPLACEMENTS {
            s = s.replace(from, to);
        }

        s
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

impl From<Config> for RandomDateTimeTransformer {
    fn from(c: Config) -> Self {
        let from = c.from;
        let to = c.to;
        let format = Self::prepare_format(c.format.as_str());

        let parsed_from = Self::parse_date(from.as_str()).unwrap();
        let parsed_to = Self::parse_date(to.as_str()).unwrap();

        Self {
            from,
            to,
            format,
            parsed_from,
            parsed_to,
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
        // let between: chrono::DateTime<Utc> = DateTimeBetween(EN, from_dt, to_dt).fake();
        let format = format_description::parse(self.format.as_str())?;
        let res: String = self.parsed_from.format(&format)?;

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
