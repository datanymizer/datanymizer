use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(from = "Config")]
pub struct Uniqueness {
    pub(crate) required: bool,
    pub(crate) try_count: Option<i64>,
}

impl Default for Uniqueness {
    fn default() -> Self {
        Self::from(FullConfig::default())
    }
}

impl From<Config> for Uniqueness {
    fn from(config: Config) -> Self {
        Self::from(match config {
            Config::Short(required) => FullConfig::from(required),
            Config::Full(full_config) => full_config,
        })
    }
}

impl From<FullConfig> for Uniqueness {
    fn from(full_config: FullConfig) -> Self {
        Self {
            required: full_config.required,
            try_count: full_config.try_count,
        }
    }
}

#[derive(Deserialize)]
#[serde(untagged)]
enum Config {
    Short(bool),
    Full(FullConfig),
}

#[derive(Deserialize)]
struct FullConfig {
    required: bool,
    try_count: Option<i64>,
}

impl Default for FullConfig {
    fn default() -> Self {
        Self {
            required: false,
            try_count: None,
        }
    }
}

impl From<bool> for FullConfig {
    fn from(required: bool) -> Self {
        Self {
            required,
            ..Self::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn deserialize(config: &str) -> Uniqueness {
        serde_yaml::from_str(config).unwrap()
    }

    #[test]
    fn short_deserialization() {
        let config = "true";
        assert_eq!(
            deserialize(config),
            Uniqueness {
                required: true,
                try_count: None
            }
        );
    }

    #[test]
    fn full_deserialization() {
        let config = r#"
required: true
try_count: 5
"#;
        assert_eq!(
            deserialize(config),
            Uniqueness {
                required: true,
                try_count: Some(5)
            }
        );
    }

    #[test]
    fn full_deserialization_no_count() {
        let config = r#"
required: true
"#;
        assert_eq!(
            deserialize(config),
            Uniqueness {
                required: true,
                try_count: None
            }
        );
    }
}
