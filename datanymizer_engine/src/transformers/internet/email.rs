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
#[serde(default)]
pub struct EmailTransformer {
    /// Email kind (`Safe`, `Free`; `Safe` is default)
    pub kind: EmailKind,
    pub uniq: Uniqueness,
}

/// Kind of email
#[derive(Serialize, Deserialize, PartialEq, Eq, Hash, Clone, Debug)]
pub enum EmailKind {
    /// Only for free email providers (e.g., "gmail.com", "yahoo.com", "hotmail.com")
    Free,
    /// Only for example domains (e.g., "example.com") - not real email addresses
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
            kind: EmailKind::Safe,
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
            EmailKind::Free => FreeEmail(EN).fake(),
            EmailKind::Safe => SafeEmail(EN).fake(),
        }
    }

    fn uniq(&self) -> &Uniqueness {
        &self.uniq
    }
}

#[cfg(test)]
mod tests {
    use super::EmailKind;
    use crate::transformers::EmailTransformer;
    use crate::{Transformer, Transformers};

    #[test]
    fn parse_config() {
        let config = "email: {}";
        let transformer: Transformers = serde_yaml::from_str(config).unwrap();
        if let Transformers::Email(transformer) = &transformer {
            assert_eq!(transformer.kind, EmailKind::Safe);
        }
        assert!(matches!(transformer, Transformers::Email(_)));
    }

    #[test]
    fn different_email_kind() {
        let config = r#"
                           email:
                             kind: Free
                           "#;
        let transformer: Transformers = serde_yaml::from_str(config).unwrap();
        if let Transformers::Email(transformer) = &transformer {
            assert_eq!(transformer.kind, EmailKind::Free);
        }
    }

    #[test]
    fn transform() {
        let transformer = EmailTransformer::new();
        let result = transformer.transform("field", "", &None).unwrap().unwrap();
        let user_and_domain: Vec<_> = result.split('@').collect();

        assert_eq!(user_and_domain.len(), 2);
        assert!(user_and_domain[1].starts_with("example."));
    }
}
