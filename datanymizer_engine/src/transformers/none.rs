use crate::transformer::{TransformResult, TransformResultHelper, Transformer, TransformContext};
use serde::{Deserialize, Serialize};

/// This transformer doing... nothing.
#[derive(Serialize, Deserialize, PartialEq, Eq, Hash, Clone, Debug)]
pub struct NoneTransformer;

impl Transformer for NoneTransformer {
    fn transform(
        &self,
        _field_name: &str,
        field_value: &str,
        _ctx: &TransformContext,
    ) -> TransformResult {
        TransformResult::present(field_value)
    }
}
