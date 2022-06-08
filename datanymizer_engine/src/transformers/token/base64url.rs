use crate::{
    transformer::{TransformContext, TransformResult, TransformResultHelper, Transformer},
    utils::rnd::rnd_chars,
};
use serde::{Deserialize, Serialize};

const DEFAULT_LENGTH: usize = 32;
const CHARS: [char; 64] = [
    '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i',
    'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z', 'A', 'B',
    'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S', 'T', 'U',
    'V', 'W', 'X', 'Y', 'Z', '-', '_',
];

/// Transformer generates random Base64Url tokens.
/// You can set a token length (default is 32) and a padding (`%3D` sequences) length.
///
/// # Examples
///
/// With defaults:
/// ```yaml
/// #...
/// rules:
///   field_name:
///     base64_token: {}
/// ```
///
/// with a custom length:
/// ```yaml
/// #...
/// rules:
///   field_name:
///     base64_token:
///       # the padding is included into the length, so we have 34 symbols and the padding (`%3D%3D`)
///       len: 36
///       pad: 2
/// ```
#[derive(Serialize, Deserialize, PartialEq, Eq, Hash, Clone, Debug)]
#[serde(default)]
pub struct Base64UrlTokenTransformer {
    /// Length
    pub len: usize,
    /// Padding
    pub pad: usize,
}

impl Default for Base64UrlTokenTransformer {
    fn default() -> Self {
        Self {
            len: DEFAULT_LENGTH,
            pad: 0,
        }
    }
}

impl Transformer for Base64UrlTokenTransformer {
    fn transform(
        &self,
        _field_name: &str,
        _field_value: &str,
        _ctx: &Option<TransformContext>,
    ) -> TransformResult {
        let padding = match self.pad {
            0 => "",
            1 => "%3D",
            2 => "%3D%3D",
            _ => panic!("Incorrect padding"),
        };
        TransformResult::present(format!(
            "{}{}",
            rnd_chars(self.len - self.pad, &CHARS),
            padding
        ))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::Transformers;

    fn transformed_value(cfg: &str) -> String {
        let transformer: Base64UrlTokenTransformer = serde_yaml::from_str(cfg).unwrap();
        transformer.transform("token", "", &None).unwrap().unwrap()
    }

    #[test]
    fn deserialize() {
        let config = "base64url_token: {}";
        let transformer: Transformers = serde_yaml::from_str(config).unwrap();

        assert_eq!(
            transformer,
            Transformers::Base64UrlToken(Base64UrlTokenTransformer::default())
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
        let value = transformed_value("len: 64");

        assert_eq!(value.len(), 64);
        for c in value.chars() {
            assert!(CHARS.contains(&c));
        }
    }

    #[test]
    fn padding() {
        let value = transformed_value("pad: 2");

        assert_eq!(value.len(), 36);
        for c in value.chars().take(30) {
            assert!(CHARS.contains(&c));
        }
        assert_eq!(&value[30..], "%3D%3D");
    }
}
