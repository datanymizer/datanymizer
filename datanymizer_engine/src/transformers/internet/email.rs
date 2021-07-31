use crate::{
    transformer::{TransformContext, UniqTransformer, Uniqueness},
    utils, Transformer, TransformerInitContext, Transformers,
};
use fake::{faker::internet::raw::*, locales::EN, Fake};
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
///
/// You can add a random alphanumeric prefix and/or suffix:
///
/// ```yaml
/// #...
/// rules:
///   field_name:
///     email:
///       # prefix length
///       prefix: 5
/// ```
///
/// ```yaml
/// #...
/// rules:
///   field_name:
///     email:
///       # suffix length
///       suffix: 5
/// ```
#[derive(Serialize, Deserialize, PartialEq, Eq, Hash, Clone, Debug, Default)]
#[serde(default)]
pub struct EmailTransformer {
    /// Email kind (`Safe`, `Free`; `Safe` is default)
    pub kind: EmailKind,
    prefix: Option<Affix>,
    suffix: Option<Affix>,
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
        field_name: &str,
        field_value: &str,
        ctx: &Option<TransformContext>,
    ) -> String {
        let mut email: String = match self.kind {
            EmailKind::Free => FreeEmail(EN).fake(),
            EmailKind::Safe => SafeEmail(EN).fake(),
        };

        if let Some(suffix) = &self.suffix {
            let parts: Vec<&str> = email.splitn(2, '@').collect();
            email = format!(
                "{}{}{}@{}",
                parts[0],
                AFFIX_SEPARATOR,
                suffix.generate(field_name, field_value, ctx),
                parts[1]
            );
        };

        if let Some(prefix) = &self.prefix {
            format!(
                "{}{}{}",
                prefix.generate(field_name, field_value, ctx),
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

    fn init(&mut self, ctx: &TransformerInitContext) {
        if let Some(Affix::Custom(tr)) = &mut self.prefix {
            tr.init(ctx);
        }

        if let Some(Affix::Custom(tr)) = &mut self.suffix {
            tr.init(ctx);
        }
    }
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Hash, Clone, Debug)]
#[serde(untagged)]
enum Affix {
    Random(usize),
    Fixed(String),
    Custom(Box<Transformers>),
}

impl Affix {
    pub fn generate(
        &self,
        field_name: &str,
        field_value: &str,
        ctx: &Option<TransformContext>,
    ) -> String {
        match self {
            Self::Random(len) => utils::rnd_chars(*len, CHARS),
            Self::Fixed(str) => str.clone(),
            Self::Custom(tr) => tr
                .transform(field_name, field_value, ctx)
                .expect("Affix generation error")
                .unwrap_or_default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transformers::FirstNameTransformer;
    use crate::{LocaleConfig, Transformer, TransformerDefaults, Transformers};

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
            let result = transformer
                .transform("field", "orig@domain.com", &None)
                .unwrap()
                .unwrap();
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
            let user_and_domain = user_and_domain("suffix: 'dev'");

            assert_eq!(user_and_domain.len(), 2);
            assert!(user_and_domain[0].ends_with("-dev"));
            assert!(user_and_domain[0].len() > 5);
        }

        #[test]
        fn both() {
            let config = r#"
                             prefix:
                               template:
                                 format: 'tpl-{{ _0 | split(pat="@") | first }}'
                             suffix: 2
                           "#;
            let user_and_domain = user_and_domain(config);

            assert_eq!(user_and_domain.len(), 2);
            assert!(user_and_domain[0].starts_with("tpl-orig-"));
            assert_eq!(
                user_and_domain[0].chars().nth_back(2).unwrap(),
                AFFIX_SEPARATOR
            );
            assert!(user_and_domain[0].len() > 13);
        }
    }

    #[test]
    fn init() {
        let config = r#"
                            prefix:
                              first_name: {}
                          "#;
        let mut transformer: EmailTransformer = serde_yaml::from_str(config).unwrap();
        let locale = LocaleConfig::RU;
        let ctx = TransformerInitContext::from_defaults(TransformerDefaults { locale });

        Transformer::init(&mut transformer, &ctx);

        assert_eq!(
            transformer.prefix.unwrap(),
            Affix::Custom(Box::new(Transformers::FirstName(FirstNameTransformer {
                locale: Some(locale)
            })))
        );
    }
}
