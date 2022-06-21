use crate::transformer::{TransformContext, TransformResult, TransformResultHelper, Transformer};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Generates UUID (http://en.wikipedia.org/wiki/Universally_unique_identifier)
#[derive(Serialize, Deserialize, PartialEq, Eq, Hash, Clone, Debug, Default)]
pub struct UuidTransformer {
    #[serde(default)]
    version: Version,
}

impl Transformer for UuidTransformer {
    fn transform(
        &self,
        _field_name: &str,
        _field_value: &str,
        _ctx: &Option<TransformContext>,
    ) -> TransformResult {
        let result = match &self.version {
            //Version::V1 => Uuid::new_v1(),
            Version::V3(domain) => Uuid::new_v3(&domain.namespace, &domain.name),
            Version::V4 => Uuid::new_v4(),
            Version::V5(domain) => Uuid::new_v5(&domain.namespace, &domain.name),
        }
        .to_string();
        TransformResult::present(&result)
    }
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Hash, Clone, Debug)]
// #[serde(tag = "version")]
enum Version {
    //V1,
    V3(Domain),
    V4,
    V5(Domain),
}

impl Default for Version {
    fn default() -> Self {
        Self::V4
    }
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Hash, Clone, Debug)]
#[serde(try_from = "DomainConfig", into = "DomainConfig")]
struct Domain {
    namespace: Uuid,
    name: Vec<u8>,
}

impl TryFrom<DomainConfig> for Domain {
    type Error = uuid::Error;

    fn try_from(c: DomainConfig) -> Result<Self, Self::Error> {
        let namespace = match c.namespace {
            Namespace::Dns => Uuid::NAMESPACE_DNS,
            Namespace::Oid => Uuid::NAMESPACE_OID,
            Namespace::Url => Uuid::NAMESPACE_URL,
            Namespace::X500 => Uuid::NAMESPACE_X500,
            Namespace::Custom(s) => Uuid::parse_str(s.as_str())?,
        };
        let name = c.name;

        Ok(Self { namespace, name })
    }
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Hash, Clone, Debug)]
struct DomainConfig {
    namespace: Namespace,
    name: Vec<u8>,
}

impl From<Domain> for DomainConfig {
    fn from(d: Domain) -> Self {
        let namespace = Namespace::Custom(d.namespace.to_string());
        let name = d.name;
        Self { namespace, name }
    }
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Hash, Clone, Debug)]
enum Namespace {
    Dns,
    Oid,
    Url,
    X500,
    Custom(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Transformers;

    fn assert_uuid(config: &str) {
        let t: Transformers = serde_yaml::from_str(config).unwrap();
        let str_uuid = t.transform("", "", &None).unwrap().unwrap();
        assert!(Uuid::parse_str(str_uuid.as_str()).is_ok());
    }

    #[test]
    fn default() {
        let config = r#"uuid: {}"#;
        assert_uuid(config);
    }

    #[test]
    fn v4() {
        let config = r#"uuid:
            version: V4"#;
        assert_uuid(config);
    }
}
