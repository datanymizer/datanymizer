mod filter;

use crate::{transformers::Transformers, Transformer, TransformerDefaults};
use anyhow::{anyhow, Result};
use config::{Config, ConfigError, File};
use serde::Deserialize;
use serde_json::Value as JsonValue;
use std::collections::HashMap;

pub use filter::{Filter, TableList};

pub type Rules = HashMap<String, Transformers>;
pub type Tables = Vec<Table>;

#[derive(Debug, Deserialize, Clone)]
pub struct Connection {
    pub database_url: Option<String>,
}

impl Connection {
    pub fn get_database_url(&self) -> String {
        self.database_url
            .clone()
            .unwrap_or_else(|| String::from(""))
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct Table {
    /// Table name
    pub name: String,
    /// Rule set for columns
    pub rules: Rules,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Settings {
    /// Source database connection
    pub source: Connection,

    /// Can be file only
    pub destination: Option<String>,

    /// Tables list with transformation rules
    pub tables: Tables,

    /// Default transformers configuration
    pub default: Option<TransformerDefaults>,

    pub filter: Option<Filter>,

    /// Global values. Visible in any template.
    /// They may be shadowed by template variables.
    pub globals: Option<HashMap<String, JsonValue>>,
}

impl Settings {
    pub fn new(path: String, database_url: String) -> Result<Self, ConfigError> {
        Self::from_source(File::with_name(&path), database_url)
    }

    fn from_source<S: 'static>(source: S, database_url: String) -> Result<Self, ConfigError>
    where
        S: config::Source + Send + Sync,
    {
        let mut s = Config::new();
        s.set("source.database_url", database_url)?;
        s.merge(source)?;

        let mut settings: Self = s.try_into()?;
        settings.preprocess();

        Ok(settings)
    }

    pub fn lookup_transformers<T>(&self, table: T, column: T) -> Option<&Transformers>
    where
        T: ToString,
    {
        let table = self.tables.iter().find(|t| t.name == table.to_string())?;
        let transformers = table.rules.get(&column.to_string())?;
        Some(&transformers)
    }

    pub fn destination(&self) -> Result<String> {
        self.destination
            .clone()
            .ok_or_else(|| anyhow!("Destination path is empty"))
    }

    fn preprocess(&mut self) {
        if let Some(defs) = &self.default {
            for table in self.tables.iter_mut() {
                for (_name, rule) in table.rules.iter_mut() {
                    rule.set_defaults(defs);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{transformers::PersonNameTransformer, LocaleConfig};
    use config::FileFormat;

    #[test]
    fn set_defaults() {
        let config = r#"
            tables:
              - name: user
                rules:
                  name:
                    person_name: {}
                  alias:
                    person_name:
                      locale: EN
            default:
              locale: RU
            "#;

        let s = Settings::from_source(File::from_str(config, FileFormat::Yaml), String::new())
            .unwrap();
        let rules = &s.tables.first().unwrap().rules;

        assert_eq!(
            rules["name"],
            Transformers::PersonName(PersonNameTransformer {
                locale: Some(LocaleConfig::RU)
            })
        );
        assert_eq!(
            rules["alias"],
            Transformers::PersonName(PersonNameTransformer {
                locale: Some(LocaleConfig::EN)
            })
        );
    }
}
