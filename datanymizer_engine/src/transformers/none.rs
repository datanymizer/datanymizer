use crate::transformer::{TransformContext, TransformResult, TransformResultHelper, Transformer};
use serde::{Deserialize, Serialize};

/// This transformer is doing... nothing.
#[derive(Serialize, Deserialize, PartialEq, Eq, Hash, Clone, Debug)]
pub struct NoneTransformer;

impl Transformer for NoneTransformer {
    fn transform(
        &self,
        _field_name: &str,
        field_value: &str,
        _ctx: &Option<TransformContext>,
    ) -> TransformResult {
        TransformResult::present(field_value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_and_transform() {
        let config = "~";
        let transformer: NoneTransformer = serde_yaml::from_str(config).unwrap();
        let value = transformer
            .transform("field", "value", &None)
            .unwrap()
            .unwrap();

        assert_eq!(value, "value");
    }
}
