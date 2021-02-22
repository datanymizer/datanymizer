use crate::transformer::{TransformContext, TransformResult, TransformResultHelper, Transformer};
use fake::{faker::internet::raw::*, locales::EN, Fake};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Eq, Hash, Clone, Debug)]
pub struct MinValue(usize);
#[derive(Serialize, Deserialize, PartialEq, Eq, Hash, Clone, Debug)]
pub struct MaxValue(usize);

/// Transformer generates random passwords.
/// You can set minimum and maximum string length
///
/// # Examples
///
/// With default values:
/// ```yaml
/// #...
/// rules:
///   field_name:
///     password: {}
/// ```
///
/// with custom length:
///
/// ```yaml
/// #...
/// rules:
///   field_name:
///     password:
///       min: 5
///       max: 10
/// ```
#[derive(Serialize, Deserialize, PartialEq, Eq, Hash, Clone, Debug)]
pub struct PasswordTransformer {
    /// Minimum password length
    #[serde(default, rename = "min")]
    pub min: MinValue,
    /// Maximum password length
    #[serde(default, rename = "max")]
    pub max: MaxValue,
}

impl Default for PasswordTransformer {
    fn default() -> Self {
        Self {
            min: MinValue::default(),
            max: MaxValue::default(),
        }
    }
}

impl Default for MinValue {
    fn default() -> Self {
        Self(8)
    }
}

impl Default for MaxValue {
    fn default() -> Self {
        Self(20)
    }
}

impl Transformer for PasswordTransformer {
    fn transform(
        &self,
        _field_name: &str,
        _field_value: &str,
        _ctx: &Option<TransformContext>,
    ) -> TransformResult {
        let range = self.min.0..self.max.0;
        let val: String = Password(EN, range).fake();
        TransformResult::present(val)
    }
}

#[cfg(test)]
mod test {
    use super::{MaxValue, MinValue};
    use crate::Transformers;

    #[test]
    fn deserialize_default_transformer() {
        let config = r#"
password: {}
"#;
        let transformer: Transformers = serde_yaml::from_str(config).unwrap();
        if let Transformers::Password(p_transformer) = transformer {
            assert_eq!(p_transformer.min, MinValue(8));
            assert_eq!(p_transformer.max, MaxValue(20));
        } else {
            assert!(false);
        }
    }

    #[test]
    fn deserialize_custom_transformer() {
        let config = r#"
password:
  min: 1
  max: 10
"#;
        let transformer: Transformers = serde_yaml::from_str(config).unwrap();

        if let Transformers::Password(p_transformer) = transformer {
            assert_eq!(p_transformer.min, MinValue(1));
            assert_eq!(p_transformer.max, MaxValue(10));
        } else {
            assert!(false);
        }
    }
}
