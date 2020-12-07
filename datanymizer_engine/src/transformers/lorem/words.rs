use crate::transformer::{Globals, TransformResult, TransformResultHelper, Transformer};
use fake::faker::lorem::raw::*;
use fake::{locales::EN, Fake};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Eq, Hash, Clone, Debug)]
pub struct WordsTransformer {
    min: Option<usize>,
    max: Option<usize>,
}

impl Default for WordsTransformer {
    fn default() -> Self {
        Self {
            min: Some(1),
            max: Some(1),
        }
    }
}

impl Transformer for WordsTransformer {
    fn transform(
        &self,
        field_name: &str,
        field_value: &str,
        _globals: &Option<Globals>,
    ) -> TransformResult {
        let min = self.min.unwrap_or(1);
        let max = self.max.unwrap_or(1);
        let mut val: Vec<String> = vec![];

        if min == 1 {
            val.extend_from_slice(&[Word(EN).fake()])
        } else {
            if min == max {
                return TransformResult::error(field_name, field_value, "MIN shouldn't be eq MAX");
            }
            let slice: Vec<String> = Words(EN, min..max).fake();
            val.extend_from_slice(&slice)
        }

        TransformResult::present(val.join(" "))
    }
}

#[cfg(test)]
mod tests {
    use super::WordsTransformer;
    use crate::Transformers;

    #[test]
    fn test_parse_config() {
        let config = r#"words: {}"#;
        let transformer: Transformers = serde_yaml::from_str(config).unwrap();
        if let Transformers::Words(WordsTransformer { min, max }) = transformer {
            assert_eq!(min, None);
            assert_eq!(max, None);
        }
    }

    #[test]
    fn test_parse_config_with_values() {
        let config = r#"words: {min: 1, max: 2}"#;
        let transformer: Transformers = serde_yaml::from_str(config).unwrap();
        if let Transformers::Words(WordsTransformer { min, max }) = transformer {
            assert_eq!(min, Some(1));
            assert_eq!(max, Some(2));
        }
    }
}
