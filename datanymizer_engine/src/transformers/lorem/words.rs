use crate::transformer::{Globals, TransformResult, TransformResultHelper, Transformer};
use fake::faker::lorem::raw::*;
use fake::{locales::EN, Fake};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Eq, Hash, Clone, Debug)]
pub struct WordsTransformer {
    min: Option<usize>,
    max: Option<usize>,
}

impl Default for WordsTransformer {
    fn default() -> Self {
        Self {
            min: Some(1),
            max: Some(1),
        }
    }
}

impl Transformer for WordsTransformer {
    fn transform(
        &self,
        field_name: &str,
        field_value: &str,
        _globals: &Option<Globals>,
    ) -> TransformResult {
        let min = self.min.unwrap_or(1);
        let max = self.max.unwrap_or(1);
        let mut val: Vec<String> = vec![];

        if min == 1 {
            val.extend_from_slice(&[Word(EN).fake()])
        } else {
            if min == max {
                return TransformResult::error(field_name, field_value, "MIN shouldn't be eq MAX");
            }
            let slice: Vec<String> = Words(EN, min..max).fake();
            val.extend_from_slice(&slice)
        }

        TransformResult::present(val.join(" "))
    }
}
