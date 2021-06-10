use crate::transformer::{TransformContext, TransformResult, TransformResultHelper, Transformer};
use serde::{Deserialize, Serialize};

use unicode_segmentation::UnicodeSegmentation;

/// Capitalize a given value (from the database, or a previous value in the pipeline)
#[derive(Serialize, Deserialize, PartialEq, Eq, Hash, Clone, Debug, Default)]
pub struct CapitalizeTransformer;

impl CapitalizeTransformer {
    pub(crate) fn capitalize(string: &str) -> String {
        string
            .split_word_bounds()
            .map(Self::capitalize_word)
            .collect::<Vec<String>>()
            .concat()
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
    use crate::{TransformResult, Transformer, Transformers};

    fn transform(value: &str) -> TransformResult {
        let config = r#"capitalize: ~"#;
        let transformer: Transformers = serde_yaml::from_str(config).unwrap();
        transformer.transform("field", value, &None)
    }

    #[test]
    fn word() {
        let expected = String::from("Value");
        assert_eq!(transform("value"), Ok(Some(expected)));
    }

    #[test]
    fn sentence() {
        let expected = String::from("Hello All People");
        assert_eq!(transform("Hello all people"), Ok(Some(expected)));
    }

    #[test]
    fn non_letter_chars() {
        let expected = String::from("Hi, Frank!");
        assert_eq!(transform("hi, frank!"), Ok(Some(expected)));
    }

    #[test]
    fn cyrillic() {
        let expected = String::from("Добрый Вечер, Ребята!");
        assert_eq!(transform("добрый вечер, ребята!"), Ok(Some(expected)));
    }
}
