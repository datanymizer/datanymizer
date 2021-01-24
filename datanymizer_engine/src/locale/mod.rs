use serde::{Deserialize, Serialize};

mod ru;
pub use ru::RU;

pub type EN = fake::locales::EN;

#[derive(Copy, Serialize, Deserialize, PartialEq, Eq, Hash, Debug, Clone)]
pub enum LocaleConfig {
    EN,
    RU,
}

impl Default for LocaleConfig {
    // We need some method to take default for all tables from config
    fn default() -> Self {
        Self::EN
    }
}

pub trait Localized {
    fn locale(&self) -> LocaleConfig;
}

pub trait LocalizedFaker<V>: Localized {
    fn fake<L: Copy + fake::locales::Data>(&self, l: L) -> V;

    fn localized_fake(&self) -> V {
        match self.locale() {
            LocaleConfig::EN => self.fake(EN {}),
            LocaleConfig::RU => self.fake(RU {}),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialization() {
        let l: LocaleConfig = serde_yaml::from_str("RU").unwrap();
        assert_eq!(l, LocaleConfig::RU);
    }
}
