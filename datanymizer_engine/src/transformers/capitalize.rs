use crate::transformer::{TransformContext, TransformResult, TransformResultHelper, Transformer};
use serde::{Deserialize, Serialize};

use unicode_segmentation::UnicodeSegmentation;

/// Capitalize inner value (from database or previews value in pipeline)
#[derive(Serialize, Deserialize, PartialEq, Eq, Hash, Clone, Debug, Default)]
pub struct CapitalizeTransformer;

impl CapitalizeTransformer {
    pub(crate) fn capitalize(string: &str) -> String {
        string
            .unicode_words()
            .map(Self::capitalize_word)
            .collect::<Vec<String>>()
            .join(" ")
    }

    fn capitalize_word(word: &str) -> String {
        word.chars()
            .enumerate()
            .map(|(i, c)| {
                if i == 0 {
                    c.to_uppercase().collect::<String>()
                } else {
                    c.to_lowercase().collect()
                }
            })
            .collect()
    }
}

impl Transformer for CapitalizeTransformer {
    fn transform(
        &self,
        _field_name: &str,
        field_value: &str,
        _ctx: &Option<TransformContext>,
    ) -> TransformResult {
        let result = Self::capitalize(field_value);
        TransformResult::present(&result)
    }
}

#[cfg(test)]
mod tests {
    use crate::{Transformer, Transformers};

    #[test]
    fn test_capitalize_word() {
        let config = r#"capitalize: ~"#;
        let transformer: Transformers = serde_yaml::from_str(config).unwrap();
        let expected = String::from("Value");
        let founded = transformer.transform("field", "value", &None);

        assert_eq!(founded, Ok(Some(expected)))
    }

    // TODO: this does not work
    // #[test]
    // fn non_letter_symbols() {
    //     let config = r#"capitalize: ~"#;
    //     let transformer: Transformers = serde_yaml::from_str(config).unwrap();
    //     let expected = String::from("Hi, Frank!");
    //     let founded = transformer.transform("field", "hi, frank!", &None);
    //
    //     assert_eq!(founded, Ok(Some(expected)))
    // }
}
