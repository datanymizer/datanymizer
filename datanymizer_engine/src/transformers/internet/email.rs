use crate::transformer::{TransformContext, UniqTransformer, Uniqueness};
use fake::faker::internet::raw::*;
use fake::locales::EN;
use fake::Fake;
use serde::{Deserialize, Serialize};

/// Transformer generates random emails
///
/// # Examples
///
/// ```yaml
/// #...
/// rules:
///   field_name:
///     email:
///       kind: Safe
/// ```
#[derive(Serialize, Deserialize, PartialEq, Eq, Hash, Clone, Debug)]
pub struct EmailTransformer {
    /// Transformation kind
    pub kind: Option<EmailKind>,

    #[serde(default)]
    pub uniq: Uniqueness,
}

/// Kind of email generator
#[derive(Serialize, Deserialize, PartialEq, Eq, Hash, Clone, Debug)]
pub enum EmailKind {
    /// Used by default
    Free,
    /// Only for "gmail.com", "yahoo.com", "hotmail.com" providers
    FreeProvider,
    /// Generates only for .com, .net, .org domains
    Safe,
}

impl EmailTransformer {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Default for EmailTransformer {
    fn default() -> Self {
        Self {
            kind: Some(EmailKind::Free),
            uniq: Uniqueness::default(),
        }
    }
}

impl UniqTransformer for EmailTransformer {
    fn do_transform(
        &self,
        _field_name: &str,
        _field_value: &str,
        _ctx: &Option<TransformContext>,
    ) -> String {
        match self.kind {
            Some(EmailKind::FreeProvider) => FreeEmailProvider(EN).fake(),
            Some(EmailKind::Free) => FreeEmail(EN).fake(),
            _ => SafeEmail(EN).fake(),
        }
    }

    fn uniq(&self) -> &Uniqueness {
        &self.uniq
    }
}

#[cfg(test)]
mod tests {
    use super::EmailKind;
    use crate::Transformers;

    #[test]
    fn test_parse_config() {
        let config = r#"email: {}"#;
        let transformer: Transformers = serde_yaml::from_str(config).unwrap();
        if let Transformers::Email(transformer) = &transformer {
            assert_eq!(transformer.kind, None);
        }
        assert!(matches!(transformer, Transformers::Email(_)));
    }

    #[test]
    fn test_different_email_kinds() {
        let config = r#"
email:
  kind: Safe
"#;
        let transformer: Transformers = serde_yaml::from_str(config).unwrap();
        if let Transformers::Email(transformer) = &transformer {
            assert_eq!(transformer.kind, Some(EmailKind::Safe));
        }
    }
}
