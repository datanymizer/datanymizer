use crate::transformer::{Globals, TransformResult, TransformResultHelper, Transformer};
use fake::Fake;
use serde::{Deserialize, Serialize};
use std::char;

#[derive(Serialize, Deserialize, PartialEq, Eq, Hash, Debug, Clone)]
pub struct PhoneTransformer {
    pub format: Option<String>,
}

impl Default for PhoneTransformer {
    fn default() -> Self {
        Self { format: None }
    }
}

impl Transformer for PhoneTransformer {
    fn transform(
        &self,
        _field_name: &str,
        _field_value: &str,
        _globals: &Option<Globals>,
    ) -> TransformResult {
        let format = self
            .format
            .clone()
            .unwrap_or_else(|| "+###########".to_string());
        let mut rng = rand::thread_rng();

        let val: String = format
            .chars()
            .map(|x| match x {
                '^' => char::from_digit((1..10).fake_with_rng::<u32, _>(&mut rng), 10).unwrap(),
                '#' => char::from_digit((0..10).fake_with_rng::<u32, _>(&mut rng), 10).unwrap(),
                other => other,
            })
            .collect();

        TransformResult::present(val)
    }
}
