use crate::{
    add_to_collector,
    transformer::{Globals, TransformResult, TransformResultHelper, Transformer},
};
use fake::Fake;
use serde::{Deserialize, Serialize};
use std::char;

const DEFAULT_FORMAT: &str = "+###########";

#[derive(Serialize, Deserialize, PartialEq, Eq, Hash, Debug, Clone)]
pub struct PhoneTransformer {
    pub format: Option<String>,

    #[serde(default)]
    pub uniq: bool,
}

impl PhoneTransformer {
    fn format(&self) -> String {
        self.format
            .clone()
            .unwrap_or_else(|| DEFAULT_FORMAT.to_string())
    }

    fn transform_simple(
        &self,
        _field_name: &str,
        _field_value: &str,
        _globals: &Option<Globals>,
    ) -> String {
        let mut rng = rand::thread_rng();

        self.format()
            .chars()
            .map(|x| match x {
                '^' => char::from_digit((1..10).fake_with_rng::<u32, _>(&mut rng), 10).unwrap(),
                '#' => char::from_digit((0..10).fake_with_rng::<u32, _>(&mut rng), 10).unwrap(),
                other => other,
            })
            .collect()
    }

    fn transform_uniq(
        &self,
        field_name: &str,
        field_value: &str,
        globals: &Option<Globals>,
    ) -> Option<String> {
        let uniq_len = self.format().len() as u32;
        let mut retry_count = if uniq_len > 10 || uniq_len < 2 {
            10
        } else {
            uniq_len.pow(uniq_len - 1)
        };
        while retry_count > 0 {
            let val = self.transform_simple(field_name, field_value, globals);
            if add_to_collector(val.clone()) {
                return Some(val);
            } else {
                retry_count -= 1;
            }
        }
        None
    }
}

impl Default for PhoneTransformer {
    fn default() -> Self {
        Self {
            format: None,
            uniq: false,
        }
    }
}

impl Transformer for PhoneTransformer {
    fn transform(
        &self,
        field_name: &str,
        field_value: &str,
        globals: &Option<Globals>,
    ) -> TransformResult {
        if self.uniq {
            match self.transform_uniq(field_name, field_value, globals) {
                Some(val) => TransformResult::present(val),
                None => TransformResult::error(field_name, field_value, "Retry limit exceeded"),
            }
        } else {
            TransformResult::present(self.transform_simple(field_name, field_value, globals))
        }
    }
}
