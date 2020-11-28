use crate::transformer::{Globals, TransformResult, TransformResultHelper, Transformer};
use serde::{Deserialize, Serialize};
use std::iter::Iterator;

#[derive(Serialize, Deserialize, PartialEq, Eq, Hash, Clone, Debug)]
pub struct PipelineTransformer<T> {
    pub pipes: Vec<T>,
}

impl<T> Default for PipelineTransformer<T> {
    fn default() -> Self {
        Self { pipes: Vec::new() }
    }
}

impl<T> Transformer for PipelineTransformer<T>
where
    T: Transformer,
{
    fn transform(
        &self,
        field_name: &str,
        field_value: &str,
        globals: &Option<Globals>,
    ) -> TransformResult {
        let res: String = self
            .pipes
            .iter()
            .fold(field_value.to_string(), |acc, pipe| {
                let transformed = pipe.transform(field_name, &acc, globals);
                if let Ok(Some(x)) = transformed {
                    x
                } else {
                    "".to_string()
                }
            });

        TransformResult::present(res)
    }
}
