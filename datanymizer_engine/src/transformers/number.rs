use crate::transformer::{Globals, TransformResult, TransformResultHelper, Transformer};
use fake::{faker::number::raw::*, locales::EN, Fake};
use rand::distributions::{Distribution, Uniform};
use serde::{Deserialize, Serialize};

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

#[derive(Serialize, Deserialize, PartialEq, Eq, Hash, Clone, Debug)]
pub struct RandomNumberTransformer {
    #[serde(default)]
    pub min: MinValue,
    #[serde(default)]
    pub max: MaxValue,
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
        }
    }
}

impl Transformer for RandomNumberTransformer {
    fn transform(
        &self,
        _field_name: &str,
        _field_value: &str,
        _globals: &Option<Globals>,
    ) -> TransformResult {
        let mut rng = rand::thread_rng();
        let num = Uniform::new_inclusive(self.min.0, self.max.0).sample(&mut rng);
        TransformResult::present(num.to_string())
    }
}
