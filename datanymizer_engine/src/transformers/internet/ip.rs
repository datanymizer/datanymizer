use crate::transformer::{TransformContext, UniqTransformer, Uniqueness};
use fake::{faker::internet::raw::*, locales::EN, Fake};
use serde::{Deserialize, Serialize};

/// IP address kind
#[derive(Serialize, Deserialize, PartialEq, Eq, Hash, Clone, Debug)]
pub enum IpKind {
    V4,
    V6,
}

/// Generates IP address by `kind` type.
///
/// # Example:
///
/// Default kind is `V4`:
///
/// ```yaml
/// #...
/// rules:
///   field_name:
///     ip: {}
/// ```
///
/// ```yaml
/// #...
/// rules:
///   field_name:
///     ip:
///       kind: V6
/// ```
#[derive(Serialize, Deserialize, PartialEq, Eq, Hash, Clone, Debug)]
pub struct IpTransformer {
    /// IP address kind (V4 or V6)
    pub kind: Option<IpKind>,

    #[serde(default)]
    pub uniq: Uniqueness,
}

impl IpTransformer {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Default for IpTransformer {
    fn default() -> Self {
        Self {
            kind: Some(IpKind::V4),
            uniq: Uniqueness::default(),
        }
    }
}

impl UniqTransformer for IpTransformer {
    fn do_transform(
        &self,
        _field_name: &str,
        _field_value: &str,
        _ctx: &Option<TransformContext>,
    ) -> String {
        match self.kind {
            Some(IpKind::V6) => IPv6(EN).fake(),
            _ => IPv4(EN).fake(),
        }
    }

    fn uniq(&self) -> &Uniqueness {
        &self.uniq
    }
}

#[cfg(test)]
mod tests {
    use super::IpKind;
    use crate::Transformers;

    #[test]
    fn test_parse_config_v4() {
        let config = r#"ip: {}"#;
        let transformer: Transformers = serde_yaml::from_str(config).unwrap();
        if let Transformers::IP(transformer) = &transformer {
            assert_eq!(transformer.kind, None);
        }
    }

    #[test]
    fn test_parse_config_v6() {
        let config = r#"
ip:
  kind: V6
"#;
        let transformer: Transformers = serde_yaml::from_str(config).unwrap();
        if let Transformers::IP(transformer) = &transformer {
            assert_eq!(transformer.kind, Some(IpKind::V6));
        }
    }
}
