use crate::transformer::{
    Globals, TransformResult, TransformResultHelper, Transformer, UniqTransformer, Uniqueness,
};
use fake::{faker::number::raw::*, locales::EN, Fake};
use rand::distributions::{Distribution, Uniform};
use serde::{Deserialize, Serialize};

/// Generates digit from `0` to `9`
///
/// # Example:
///
/// ```yaml
/// #...
/// rules:
///   field_name:
///     digit: ~
/// ```
#[derive(Serialize, Deserialize, PartialEq, Eq, Hash, Debug, Clone)]
pub struct DigitTransformer;

impl Transformer for DigitTransformer {
    fn transform(
        &self,
        _field_name: &str,
        _field_value: &str,
        _globals: &Option<Globals>,
    ) -> TransformResult {
        let val: String = Digit(EN).fake();
        TransformResult::present(val)
    }
}

#[derive(Serialize, Deserialize, PartialEq, Eq, PartialOrd, Hash, Clone, Debug)]
pub struct MinValue(usize);
#[derive(Serialize, Deserialize, PartialEq, Eq, PartialOrd, Hash, Clone, Debug)]
pub struct MaxValue(usize);

/// Generates random number from `min` to `max` range.
///
/// # Example:
///
/// with default values:
///
/// ```yaml
/// #...
/// rules:
///   field_name:
///     random_num: {}
/// ```
///
/// or with custom range:
///
/// ```yaml
/// #...
/// rules:
///   field_name:
///     random_num:
///       min: 10
///       max: 20
/// ```
#[derive(Serialize, Deserialize, PartialEq, Eq, Hash, Clone, Debug)]
pub struct RandomNumberTransformer {
    #[serde(default)]
    pub min: MinValue,

    #[serde(default)]
    pub max: MaxValue,

    #[serde(default)]
    pub uniq: Uniqueness,
}

impl Default for MinValue {
    fn default() -> Self {
        Self(usize::MIN)
    }
}

impl Default for MaxValue {
    fn default() -> Self {
        Self(usize::MAX)
    }
}

impl Default for RandomNumberTransformer {
    fn default() -> Self {
        Self {
            min: MinValue::default(),
            max: MaxValue::default(),
            uniq: Uniqueness::default(),
        }
    }
}

impl UniqTransformer for RandomNumberTransformer {
    fn do_transform(
        &self,
        _field_name: &str,
        _field_value: &str,
        _globals: &Option<Globals>,
    ) -> String {
        let mut rng = rand::thread_rng();
        Uniform::new_inclusive(self.min.0, self.max.0)
            .sample(&mut rng)
            .to_string()
    }

    fn uniq(&self) -> &Uniqueness {
        &self.uniq
    }
}
