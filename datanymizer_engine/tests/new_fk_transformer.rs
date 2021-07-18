// Test the faker-based transformer implementation from docs/new_fk_transformer.md

use fake::{locales::Data, Dummy};
use rand::Rng;

// Mock faker
struct Passport<L>(pub L);

impl<L: Data> Dummy<Passport<L>> for String {
    fn dummy_with_rng<R: Rng + ?Sized>(_: &Passport<L>, _rng: &mut R) -> Self {
        String::from("1234567")
    }
}

use datanymizer_engine::{
    FkTransformer, LocaleConfig, Localized, LocalizedFaker, TransformContext, TransformResult,
    Transformer, TransformerDefaults, TransformerInitContext,
};
use fake::Fake;
use serde::{Deserialize, Serialize};

// Test transformer
#[derive(Default, Serialize, Deserialize, PartialEq, Eq, Hash, Debug, Clone)]
#[serde(default)]
pub struct PassportTransformer {
    pub locale: Option<LocaleConfig>,
}

impl Localized for PassportTransformer {
    fn locale(&self) -> Option<LocaleConfig> {
        self.locale
    }

    fn set_locale(&mut self, l: Option<LocaleConfig>) {
        self.locale = l;
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
        _ctx: &Option<TransformContext>,
    ) -> TransformResult {
        self.transform_with_faker()
    }

    fn init(&mut self, ctx: &TransformerInitContext) {
        self.set_defaults_for_faker(&ctx.defaults);
    }
}

#[test]
fn transform() {
    let t = PassportTransformer::default();
    let value = t.transform("table.field", "value", &None).unwrap().unwrap();
    assert_eq!(value, "1234567");
}

#[test]
fn set_defaults() {
    let mut t = PassportTransformer::default();
    assert_eq!(t.locale(), None);

    t.init(&TransformerInitContext::from_defaults(
        TransformerDefaults {
            locale: LocaleConfig::RU,
        },
    ));
    assert_eq!(t.locale(), Some(LocaleConfig::RU));
}
