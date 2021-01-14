mod filter_config;

use crate::transformers::Transformers;
use anyhow::{anyhow, Result};
use config::{Config, ConfigError, File};
use filter_config::FilterDetails;
use serde::Deserialize;
use serde_json::Value as JsonValue;
use std::collections::HashMap;

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

/// Filter for include or exclude tables
#[derive(Clone, Debug, Deserialize)]
pub enum Filter {
    #[serde(rename = "only", alias = "include")]
    Only(FilterDetails),
    #[serde(rename = "except", alias = "exclude")]
    Except(FilterDetails),
}

#[derive(Debug, Deserialize, Clone)]
pub struct Settings {
    /// Source database connection
    pub source: Connection,

    /// Can be file only
    pub destination: Option<String>,

    /// Tables list with transformation rules
    pub tables: Tables,

    pub filter: Option<Filter>,

    /// Global values. Visible in any template.
    /// They may be shadowed by template variables.
    pub globals: Option<HashMap<String, JsonValue>>,
}

impl Settings {
    pub fn new(path: String, database_url: String) -> Result<Self, ConfigError> {
        let mut s = Config::new();
        s.set("source.database_url", database_url)?;
        s.merge(File::with_name(&path))?;

        s.try_into()
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
}
