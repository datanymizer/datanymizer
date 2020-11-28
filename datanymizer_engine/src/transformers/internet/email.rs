use crate::transformer::{Globals, TransformResult, TransformResultHelper, Transformer};
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
        }
    }
}

impl Transformer for EmailTransformer {
    fn transform(
        &self,
        _field_name: &str,
        _field_value: &str,
        _globals: &Option<Globals>,
    ) -> TransformResult {
        let val: String = match self.kind {
            Some(EmailKind::FreeProvider) => FreeEmailProvider(EN).fake(),
            Some(EmailKind::Free) => FreeEmail(EN).fake(),
            _ => SafeEmail(EN).fake(),
        };

        TransformResult::present(val)
    }
}
