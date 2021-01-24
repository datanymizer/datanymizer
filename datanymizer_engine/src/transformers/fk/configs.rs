use serde::{Deserialize, Serialize};
use std::ops::Range;

#[derive(Serialize, Deserialize, PartialEq, Eq, Hash, Debug, Clone)]
#[serde(default)]
pub struct RatioConfig {
    pub ratio: u8,
}

impl Default for RatioConfig {
    fn default() -> Self {
        Self { ratio: 50 }
    }
}

#[derive(Serialize, Default, Deserialize, PartialEq, Eq, Hash, Debug, Clone)]
pub struct EmptyConfig;

#[derive(Serialize, Deserialize, PartialEq, Eq, Hash, Debug, Clone)]
pub struct RangeConfig {
    pub min: usize,
    pub max: usize,
}

impl RangeConfig {
    pub fn range(&self) -> Range<usize> {
        self.min..self.max + 1
    }
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Hash, Debug, Clone)]
#[serde(default)]
pub struct LenConfig {
    pub len: RangeConfig,
}

impl Default for LenConfig {
    fn default() -> Self {
        Self {
            len: RangeConfig { min: 10, max: 20 },
        }
    }
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Hash, Debug, Clone)]
#[serde(default)]
pub struct CountConfig {
    pub count: RangeConfig,
}

impl Default for CountConfig {
    fn default() -> Self {
        Self {
            count: RangeConfig { min: 5, max: 10 },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod deserialization {
        use super::*;

        #[test]
        fn ratio() {
            let cfg: RatioConfig = serde_yaml::from_str("ratio: 30").unwrap();
            assert_eq!(cfg, RatioConfig { ratio: 30 });
        }

        #[test]
        fn empty() {
            let cfg: EmptyConfig = serde_yaml::from_str("~").unwrap();
            assert_eq!(cfg, EmptyConfig);
        }

        #[test]
        fn len() {
            let cfg = r#"
                len:
                  min: 1
                  max: 5
                "#;
            let cfg: LenConfig = serde_yaml::from_str(cfg).unwrap();
            assert_eq!(
                cfg,
                LenConfig {
                    len: RangeConfig { min: 1, max: 5 }
                }
            );
        }

        #[test]
        fn count() {
            let cfg = r#"
                count:
                  min: 2
                  max: 5
                "#;
            let cfg: CountConfig = serde_yaml::from_str(cfg).unwrap();
            assert_eq!(
                cfg,
                CountConfig {
                    count: RangeConfig { min: 2, max: 5 }
                }
            );
        }
    }
}
