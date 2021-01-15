use serde::Deserialize;

/// Filter for include or exclude tables
#[derive(Clone, Debug, Deserialize)]
#[serde(from = "Config")]
pub struct Filter {
    pub schema: Option<TableList>,
    pub data: Option<TableList>,
}

impl From<Config> for Filter {
    fn from(config: Config) -> Self {
        Self::from(match config {
            Config::Short(list) => FullConfig::from(list),
            Config::Full(full_config) => full_config,
        })
    }
}

impl From<FullConfig> for Filter {
    fn from(full_config: FullConfig) -> Self {
        Self {
            schema: full_config.schema,
            data: full_config.data,
        }
    }
}

impl Filter {
    #[allow(clippy::ptr_arg)]
    pub fn filter_schema(&self, table: &String) -> bool {
        Self::filter(&self.schema, table)
    }

    #[allow(clippy::ptr_arg)]
    pub fn filter_data(&self, table: &String) -> bool {
        Self::filter(&self.data, table)
    }

    #[allow(clippy::ptr_arg)]
    fn filter(list: &Option<TableList>, table: &String) -> bool {
        if let Some(l) =list {
            l.filter(&table)
        } else {
            true
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
pub enum TableList {
    #[serde(rename = "only", alias = "include")]
    Only(Vec<String>),
    #[serde(rename = "except", alias = "exclude")]
    Except(Vec<String>),
}

impl TableList {
    #[allow(clippy::ptr_arg)]
    pub fn filter(&self, table: &String) -> bool {
        match self {
            Self::Only(tables) => tables.contains(table),
            Self::Except(tables) => !tables.contains(table),
        }
    }

    pub fn tables(&self) -> &Vec<String> {
        match self {
            Self::Only(tables) => tables,
            Self::Except(tables) => tables,
        }
    }
}

#[derive(Deserialize)]
#[serde(untagged)]
enum Config {
    Short(TableList),
    Full(FullConfig),
}

#[derive(Deserialize)]
struct FullConfig {
    schema: Option<TableList>,
    data: Option<TableList>,
}

impl From<TableList> for FullConfig {
    fn from(list: TableList) -> Self {
        Self {
            schema: None,
            data: Some(list),
        }
    }
}
