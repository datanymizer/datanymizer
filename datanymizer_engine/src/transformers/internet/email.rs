use crate::{
    transformer::{TransformContext, UniqTransformer, Uniqueness},
    utils,
};
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
#[derive(Serialize, Deserialize, PartialEq, Eq, Hash, Clone, Debug, Default)]
#[serde(default)]
pub struct EmailTransformer {
    /// Email kind (`Safe`, `Free`; `Safe` is default)
    pub kind: EmailKind,
    pub prefix: usize,
    pub suffix: usize,
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

impl Default for EmailKind {
    fn default() -> Self {
        Self::Safe
    }
}

const CHARS: &[char] = &[
    'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's',
    't', 'u', 'v', 'w', 'x', 'y', 'z', '0', '1', '2', '3', '4', '5', '6', '7', '8', '9',
];

const AFFIX_SEPARATOR: char = '-';

impl EmailTransformer {
    pub fn new() -> Self {
        Self::default()
    }
}

impl UniqTransformer for EmailTransformer {
    fn do_transform(
        &self,
        _field_name: &str,
        _field_value: &str,
        _ctx: &Option<TransformContext>,
    ) -> String {
        let mut email: String = match self.kind {
            EmailKind::Free => FreeEmail(EN).fake(),
            EmailKind::Safe => SafeEmail(EN).fake(),
        };

        if self.suffix > 0 {
            let parts: Vec<&str> = email.splitn(2, '@').collect();
            email = format!(
                "{}{}{}@{}",
                parts[0],
                AFFIX_SEPARATOR,
                utils::rnd_chars(self.suffix, CHARS),
                parts[1]
            );
        };

        if self.prefix > 0 {
            format!(
                "{}{}{}",
                utils::rnd_chars(self.prefix, CHARS),
                AFFIX_SEPARATOR,
                email
            )
        } else {
            email
        }
    }

    fn uniq(&self) -> &Uniqueness {
        &self.uniq
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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

    mod transform {
        use super::*;

        fn user_and_domain(config: &str) -> Vec<String> {
            let transformer: EmailTransformer = serde_yaml::from_str(config).unwrap();
            let result = transformer.transform("field", "", &None).unwrap().unwrap();
            result.split('@').map(String::from).collect()
        }

        #[test]
        fn default() {
            let user_and_domain = user_and_domain("{}");

            assert_eq!(user_and_domain.len(), 2);
            assert!(user_and_domain[1].starts_with("example."));
        }

        #[test]
        fn prefix() {
            let user_and_domain = user_and_domain("prefix: 5");

            assert_eq!(user_and_domain.len(), 2);
            assert_eq!(user_and_domain[0].chars().nth(5).unwrap(), AFFIX_SEPARATOR);
            assert!(user_and_domain[0].len() > 7);
        }

        #[test]
        fn suffix() {
            let user_and_domain = user_and_domain("suffix: 4");

            assert_eq!(user_and_domain.len(), 2);
            assert_eq!(
                user_and_domain[0].chars().nth_back(4).unwrap(),
                AFFIX_SEPARATOR
            );
            assert!(user_and_domain[0].len() > 6);
        }

        #[test]
        fn both() {
            let config = r#"
                             prefix: 3
                             suffix: 2
                           "#;
            let user_and_domain = user_and_domain(config);

            assert_eq!(user_and_domain.len(), 2);
            assert_eq!(user_and_domain[0].chars().nth(3).unwrap(), AFFIX_SEPARATOR);
            assert_eq!(
                user_and_domain[0].chars().nth_back(2).unwrap(),
                AFFIX_SEPARATOR
            );
            assert!(user_and_domain[0].len() > 8);
        }
    }
}
