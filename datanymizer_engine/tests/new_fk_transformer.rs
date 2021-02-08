// Test the faker-based transformer implementation from docs/new_fk_transformer.md

use fake::{locales::Data, Dummy, Fake};
use rand::Rng;
use serde::{Deserialize, Serialize};

use datanymizer_engine::{
    FkTransformer, Globals, LocaleConfig, Localized, LocalizedFaker, TransformResult, Transformer,
};

// Mock faker
struct Passport<L>(pub L);

impl<L: Data> Dummy<Passport<L>> for String {
    fn dummy_with_rng<R: Rng + ?Sized>(_: &Passport<L>, _rng: &mut R) -> Self {
        String::from("1234567")
    }
}

#[derive(Default, Serialize, Deserialize, PartialEq, Eq, Hash, Debug, Clone)]
#[serde(default)]
pub struct PassportTransformer {
    pub locale: LocaleConfig,
}

impl Localized for PassportTransformer {
    fn locale(&self) -> LocaleConfig {
        self.locale
    }
}

impl LocalizedFaker<String> for PassportTransformer {
    fn fake<L: Copy + fake::locales::Data>(&self, l: L) -> String {
        Passport(l).fake()
    }
}

impl FkTransformer<String> for PassportTransformer {}

impl Transformer for PassportTransformer {
    fn transform(
        &self,
        _field_name: &str,
        _field_value: &str,
        _globals: &Option<Globals>,
    ) -> TransformResult {
        self.transform_with_faker()
    }
}

#[test]
fn transform() {
    let t = PassportTransformer::default();
    let value = t.transform("table.field", "value", &None).unwrap().unwrap();
    assert_eq!(value, "1234567");
}
