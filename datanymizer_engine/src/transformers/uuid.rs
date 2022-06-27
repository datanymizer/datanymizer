use crate::transformer::{TransformContext, TransformResult, TransformResultHelper, Transformer};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Generates random UUIDs (http://en.wikipedia.org/wiki/Universally_unique_identifier)
/// It uses the UUID version 4 algorithm.
///
/// # Example:
///
/// ```yaml
/// #...
/// rules:
///   field_name:
///     uuid: ~
/// ```
#[derive(Serialize, Deserialize, PartialEq, Eq, Hash, Clone, Debug, Default)]
pub struct UuidTransformer;

impl Transformer for UuidTransformer {
    fn transform(
        &self,
        _field_name: &str,
        _field_value: &str,
        _ctx: &Option<TransformContext>,
    ) -> TransformResult {
        TransformResult::present(&Uuid::new_v4().to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Transformers;

    #[test]
    fn transform() {
        let config = r#"uuid: ~"#;
        let t: Transformers = serde_yaml::from_str(config).unwrap();

        let uuid1 = t.transform("", "", &None).unwrap().unwrap();
        assert!(Uuid::parse_str(uuid1.as_str()).is_ok());

        let uuid2 = t.transform("", "", &None).unwrap().unwrap();
        assert!(Uuid::parse_str(uuid2.as_str()).is_ok());

        assert_ne!(uuid1, uuid2);
    }
}
