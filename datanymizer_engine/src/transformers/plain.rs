use crate::{transformer::TransformContext, TransformResult, Transformer};
use serde::{Deserialize, Serialize};

/// Just outputs a fixed string (plain text).
///
/// # Example:
///
/// #...
/// rules:
///   field_name:
///     plain: "some text"
/// ```
#[derive(Serialize, Deserialize, PartialEq, Eq, Hash, Clone, Debug)]
pub struct PlainTransformer(String);

impl Transformer for PlainTransformer {
    fn transform(
        &self,
        _field_name: &str,
        _field_value: &str,
        _ctx: &Option<TransformContext>,
    ) -> TransformResult {
        Ok(Some(self.0.clone()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::EnumWrapper;
    use crate::Transformers;

    #[test]
    fn parse_and_transform() {
        let cfg = "plain: 'some text'";
        let transformer: Transformers = EnumWrapper::parse(cfg).unwrap();
        let result = transformer.transform("field", "", &None).unwrap().unwrap();

        assert_eq!(result, "some text");
    }
}
