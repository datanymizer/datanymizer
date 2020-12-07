use crate::transformer::{Globals, TransformResult, TransformResultHelper, Transformer};
use fake::{faker::internet::raw::*, locales::EN, Fake};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Eq, Hash, Clone, Debug)]
pub enum IpKind {
    V4,
    V6,
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Hash, Clone, Debug)]
pub struct IpTransformer {
    pub kind: Option<IpKind>,
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
        }
    }
}

impl Transformer for IpTransformer {
    fn transform(
        &self,
        _field_name: &str,
        _field_value: &str,
        _globals: &Option<Globals>,
    ) -> TransformResult {
        let val: String = match self.kind {
            Some(IpKind::V6) => IPv6(EN).fake(),
            _ => IPv4(EN).fake(),
        };

        TransformResult::present(val)
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
