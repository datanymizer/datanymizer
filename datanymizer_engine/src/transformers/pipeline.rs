use crate::transformer::{
    TransformContext, TransformResult, TransformResultHelper, Transformer, TransformerDefaults,
};
use serde::{Deserialize, Serialize};
use std::iter::Iterator;

/// You can use `pipelines` with complicated rules to generate more difficult values.
/// You can use any transformer as steps (as well as a pipelines to).
///
/// # Example:
///
/// ```yaml
/// #...
/// rules:
///   field_name:
///     pipeline:
///       pipes:
///         - email: {}
///         - capitalize: ~
/// ```
/// The pipes will be executed in the order in which they are specified in the config.
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
        ctx: &Option<TransformContext>,
    ) -> TransformResult {
        let res: String = self
            .pipes
            .iter()
            .fold(field_value.to_string(), |acc, pipe| {
                let transformed = pipe.transform(field_name, &acc, ctx);
                if let Ok(Some(x)) = transformed {
                    x
                } else {
                    "".to_string()
                }
            });

        TransformResult::present(res)
    }

    fn set_defaults(&mut self, defaults: &TransformerDefaults) {
        for t in &mut self.pipes {
            t.set_defaults(defaults);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        transformers::{CapitalizeTransformer, FirstNameTransformer, LastNameTransformer},
        LocaleConfig, Transformers,
    };

    #[test]
    fn set_defaults() {
        let mut t = PipelineTransformer {
            pipes: vec![
                Transformers::FirstName(FirstNameTransformer::default()),
                Transformers::LastName(LastNameTransformer {
                    locale: Some(LocaleConfig::ZH_TW),
                }),
                Transformers::Capitalize(CapitalizeTransformer),
            ],
        };
        t.set_defaults(&TransformerDefaults {
            locale: LocaleConfig::RU,
        });

        assert!(
            matches!(&t.pipes[0], Transformers::FirstName(t) if t.locale == Some(LocaleConfig::RU))
        );
        assert!(
            matches!(&t.pipes[1], Transformers::LastName(t) if t.locale == Some(LocaleConfig::ZH_TW))
        );
    }
}
