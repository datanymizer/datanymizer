use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
#[serde(from = "Config")]
pub struct FilterDetails {
    pub schema: Vec<String>,
    pub data: Vec<String>,
}

impl From<Config> for FilterDetails {
    fn from(config: Config) -> Self {
        Self::from(match config {
            Config::Short(tables) => FullConfig::from(tables),
            Config::Full(full_config) => full_config,
        })
    }
}

impl From<FullConfig> for FilterDetails {
    fn from(full_config: FullConfig) -> Self {
        Self {
            schema: full_config.schema,
            data: full_config.data,
        }
    }
}

#[derive(Deserialize)]
#[serde(untagged)]
enum Config {
    Short(Vec<String>),
    Full(FullConfig),
}

#[derive(Deserialize)]
struct FullConfig {
    schema: Vec<String>,
    data: Vec<String>,
}

impl From<Vec<String>> for FullConfig {
    fn from(tables: Vec<String>) -> Self {
        Self {
            schema: Vec::new(),
            data: tables,
        }
    }
}
