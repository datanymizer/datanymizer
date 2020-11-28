use crate::transformer::{Globals, TransformResult, TransformResultHelper, Transformer};
use fake::faker::address::raw::*;
use fake::locales::EN;
use fake::Fake;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Eq, Hash, Clone, Debug, Default)]
pub struct CityTransformer;

impl Transformer for CityTransformer {
    fn transform(
        &self,
        _field_name: &str,
        _field_value: &str,
        _globals: &Option<Globals>,
    ) -> TransformResult {
        let val: String = CityName(EN).fake();
        TransformResult::present(val)
    }
}
