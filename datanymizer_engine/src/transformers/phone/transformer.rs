use super::deserialize_phone_format;
use super::phone_format::PhoneFormat;
use crate::{
    add_to_collector,
    transformer::{Globals, TransformResult, TransformResultHelper, Transformer},
};
use fake::Fake;
use serde::{Deserialize, Serialize};
use std::char;

#[derive(Serialize, Deserialize, PartialEq, Eq, Hash, Debug, Clone)]
pub struct PhoneTransformer {
    #[serde(deserialize_with = "deserialize_phone_format", default)]
    pub format: Option<PhoneFormat>,

    #[serde(default)]
    pub uniq: bool,
}

impl PhoneTransformer {
    fn phone_format(&self) -> PhoneFormat {
        self.format.clone().unwrap_or_default()
    }

    fn transform_simple(
        &self,
        _field_name: &str,
        _field_value: &str,
        _globals: &Option<Globals>,
    ) -> String {
        let mut rng = rand::thread_rng();

        self.phone_format()
            .source_format
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
        let mut retry_count = self.phone_format().invariants();
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
                None => {
                    let phone_format = self.phone_format();
                    let message = format!(
                        "field: `{}` with retry limit: `{}` exceeded for format: `{}`",
                        field_name,
                        phone_format.invariants(),
                        phone_format.source_format
                    );
                    TransformResult::error(field_name, field_value, &message)
                }
            }
        } else {
            TransformResult::present(self.transform_simple(field_name, field_value, globals))
        }
    }
}
