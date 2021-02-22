use super::deserialize_phone_format;
use super::phone_format::PhoneFormat;
use crate::transformer::{TransformContext, UniqTransformer, Uniqueness};
use fake::Fake;
use serde::{Deserialize, Serialize};
use std::char;

/// Generates phone numbers by template
///
/// # Example:
///
/// ```yaml
/// #...
/// rules:
///   field_name:
///     phone:
///       format: +^##########
/// ```
/// where:
/// * `format` - you can specify format for you phone number
/// * `#` - any digit from `0` to `9`
/// * `^` - any digit from `1` to `9`
/// Also, you can use any other symbols in `format`, like: `^##-00-### (##-##)`
///
/// If you want to generate unique phone numbers, use this option:
///
/// ```yaml
/// #...
/// rules:
///   field_name:
///     phone:
///       format: +^12345678##
///       uniq: true
/// ```
/// The transformer will collect information about generated numbers and check their uniqueness.
/// If such a number already exists in the list, then the transformer will try to generate the value again.
/// The number of attempts is limited by the number of available invariants based on the `format`.
#[derive(Serialize, Deserialize, PartialEq, Eq, Hash, Debug, Clone)]
pub struct PhoneTransformer {
    #[serde(deserialize_with = "deserialize_phone_format", default)]
    pub format: Option<PhoneFormat>,

    #[serde(default)]
    pub uniq: Uniqueness,
}

impl PhoneTransformer {
    fn phone_format(&self) -> PhoneFormat {
        self.format.clone().unwrap_or_default()
    }
}

impl Default for PhoneTransformer {
    fn default() -> Self {
        Self {
            format: None,
            uniq: Uniqueness::default(),
        }
    }
}

impl UniqTransformer for PhoneTransformer {
    fn do_transform(
        &self,
        _field_name: &str,
        _field_value: &str,
        _ctx: &Option<TransformContext>,
    ) -> String {
        let mut rng = rand::thread_rng();

        self.phone_format()
            .source_format
            .chars()
            .map(|x| match x {
                '^' => char::from_digit((1..10).fake_with_rng::<u32, _>(&mut rng), 10).unwrap(),
                '#' => char::from_digit((0..10).fake_with_rng::<u32, _>(&mut rng), 10).unwrap(),
                other => other,
            })
            .collect()
    }

    fn uniq(&self) -> &Uniqueness {
        &self.uniq
    }

    fn default_try_count(&self) -> i64 {
        self.phone_format().invariants()
    }

    fn try_limit_message(&self, field_name: &str) -> String {
        format!(
            "field: `{}` with retry limit: `{}` exceeded for format: `{}`",
            field_name,
            self.try_count(),
            self.phone_format().source_format
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::{transformer::TransformResult, Transformer, Transformers};

    #[test]
    fn parse_config_to_phone_transformer() {
        let config = r#"
        phone:
            format: +123456789#
        "#;

        let transformer: Transformers = serde_yaml::from_str(config).unwrap();
        assert!(matches!(transformer, Transformers::Phone(_)));
    }

    #[test]
    fn generate_uniq_phone_number() {
        let config = r#"
        phone:
            format: +123456789#
            uniq: true
        "#;

        let transformer: Transformers = serde_yaml::from_str(config).unwrap();

        let val1 = transformer.transform("field", "value", &None);
        let val2 = transformer.transform("field", "value", &None);

        assert_ne!(val1, val2);
    }

    #[test]
    #[warn(unused_doc_comments)]
    fn test_max_invariants_of_uniq_phones() {
        let config = r#"
        phone:
            format: +123456789#
            uniq: true
        "#;
        let transformer: Transformers = serde_yaml::from_str(config).unwrap();

        let mut phones: Vec<TransformResult> = vec![];
        for _ in 0..5 {
            phones.push(transformer.transform("field", "value", &None));
        }

        assert!(phones.iter().any(|x| x.is_ok()))
    }
}
