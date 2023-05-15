use crate::{
    transformer::{TransformContext, TransformResult, TransformResultHelper, Transformer},
    utils::rnd::rnd_chars,
};
use serde::{Deserialize, Serialize};

const DEFAULT_LENGTH: usize = 32;
const CHARS: [char; 16] = [
    '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'a', 'b', 'c', 'd', 'e', 'f',
];

/// Transformer generates random hex tokens.
/// You can set a token length (default is 32)
///
/// # Examples
///
/// With defaults:
/// ```yaml
/// #...
/// rules:
///   field_name:
///     hex_token: {}
/// ```
///
/// with a custom length:
/// ```yaml
/// #...
/// rules:
///   field_name:
///     hex_token:
///       len: 128
/// ```
#[derive(Serialize, Deserialize, PartialEq, Eq, Hash, Clone, Debug)]
#[serde(default)]
pub struct HexTokenTransformer {
    /// Length
    pub len: usize,
}

impl Default for HexTokenTransformer {
    fn default() -> Self {
        Self {
            len: DEFAULT_LENGTH,
        }
    }
}

impl Transformer for HexTokenTransformer {
    fn transform(
        &self,
        _field_name: &str,
        _field_value: &str,
        _ctx: &Option<TransformContext>,
    ) -> TransformResult {
        TransformResult::present(rnd_chars(self.len, &CHARS))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{utils::EnumWrapper, Transformers};

    fn transformed_value(cfg: &str) -> String {
        let transformer: HexTokenTransformer = serde_yaml::from_str(cfg).unwrap();
        transformer.transform("token", "", &None).unwrap().unwrap()
    }

    #[test]
    fn deserialize() {
        let config = "hex_token: {}";
        let transformer: Transformers = EnumWrapper::parse(config).unwrap();

        assert_eq!(
            transformer,
            Transformers::HexToken(HexTokenTransformer::default())
        );
    }

    #[test]
    fn default() {
        let value = transformed_value("{}");

        assert_eq!(value.len(), DEFAULT_LENGTH);
        for c in value.chars() {
            assert!(CHARS.contains(&c));
        }
    }

    #[test]
    fn custom_length() {
        let value = transformed_value("len: 50");

        assert_eq!(value.len(), 50);
        for c in value.chars() {
            assert!(CHARS.contains(&c));
        }
    }
}
