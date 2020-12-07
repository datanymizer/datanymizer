use crate::transformer::{Globals, TransformResult, TransformResultHelper, Transformer};
use fake::faker::name::raw::*;
use fake::locales::EN;
use fake::Fake;
use serde::{Deserialize, Serialize};

/// First name transformer
///
/// # Example:
///
/// ```yaml
/// #...
/// rules:
///   field_name:
///     first_name: ~
/// ```
#[derive(Serialize, Deserialize, PartialEq, Eq, Hash, Clone, Debug, Default)]
pub struct FirstNameTransformer;

/// Last name transformer
///
/// # Example:
///
/// ```yaml
/// #...
/// rules:
///   field_name:
///     last_name: ~
/// ```
#[derive(Serialize, Deserialize, PartialEq, Eq, Hash, Clone, Debug, Default)]
pub struct LastNameTransformer;

impl Transformer for FirstNameTransformer {
    fn transform(
        &self,
        _field_name: &str,
        _field_value: &str,
        _globals: &Option<Globals>,
    ) -> TransformResult {
        let val: String = FirstName(EN).fake();
        TransformResult::present(val)
    }
}

impl Transformer for LastNameTransformer {
    fn transform(
        &self,
        _field_name: &str,
        _field_value: &str,
        _globals: &Option<Globals>,
    ) -> TransformResult {
        let val: String = LastName(EN).fake();
        TransformResult::present(val)
    }
}
