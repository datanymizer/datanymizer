mod filter;
mod table;

use crate::{
    transformer::{TransformerDefaults, TransformerInitContext},
    transformers::Transformers,
    Transformer,
};
use anyhow::{anyhow, Result};
use config::{Config, ConfigError, File, FileFormat};
use serde::Deserialize;
use serde_json::Value as JsonValue;
use std::collections::HashMap;

pub use filter::{Filter, TableList};
pub use table::{Query, Table};

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

type TransformList = Vec<(String, Transformers)>;

#[derive(Debug, Deserialize, Clone)]
pub struct Settings {
    /// Source database connection
    pub source: Connection,

    /// Can be file only
    pub destination: Option<String>,

    /// Tables list with transformation rules
    pub tables: Tables,

    /// Default transformers configuration
    #[serde(default)]
    pub default: TransformerDefaults,

    pub filter: Option<Filter>,

    /// Global values. Visible in any template.
    /// They may be shadowed by template variables.
    pub globals: Option<HashMap<String, JsonValue>>,

    #[serde(skip)]
    transform_map: Option<HashMap<String, TransformList>>,
}

impl Settings {
    pub fn new(path: String, database_url: String) -> Result<Self, ConfigError> {
        Self::from_source(File::with_name(&path), database_url)
    }

    pub fn from_yaml(config: &str, database_url: String) -> Result<Self, ConfigError> {
        Self::from_source(File::from_str(config, FileFormat::Yaml), database_url)
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

    pub fn transformers_for(&self, table: &str) -> Option<&TransformList> {
        if let Some(m) = &self.transform_map {
            m.get(table)
        } else {
            panic!("No transform map");
        }
    }

    pub fn get_table(&self, name: &str) -> Option<&Table> {
        self.tables.iter().find(|t| t.name == name)
    }

    pub fn find_table(&self, names: &[&str]) -> Option<&Table> {
        for name in names {
            let table = self.get_table(name);
            if table.is_some() {
                return table;
            }
        }
        None
    }

    pub fn destination(&self) -> Result<String> {
        self.destination
            .clone()
            .ok_or_else(|| anyhow!("Destination path is empty"))
    }

    fn preprocess(&mut self) {
        let init_ctx = TransformerInitContext::from_defaults(self.default.clone());

        for table in self.tables.iter_mut() {
            for (_name, rule) in table.rules.iter_mut() {
                rule.init(&init_ctx);
            }
        }

        self.fill_transform_map();
    }

    fn fill_transform_map(&mut self) {
        let mut map = HashMap::with_capacity(self.tables.len());
        for table in &self.tables {
            map.insert(table.name.clone(), table.transform_list());
        }

        self.transform_map = Some(map);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{transformers::PersonNameTransformer, LocaleConfig};

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

        let s = Settings::from_yaml(config, String::new()).unwrap();
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

    #[test]
    fn find_table() {
        let config = r#"
            tables:
              - name: companies
                rules:
                  name:
                    company_name: {}
              - name: users
                rules:
                  name:
                    person_name: {}
              - name: other_schema.users
                rules:
                  other_name:
                    person_name: {}
            "#;
        let s = Settings::from_yaml(config, String::new()).unwrap();

        let t = s.find_table(&["some_table"]);
        assert!(t.is_none());

        let t = s.find_table(&["some_table", "users"]);
        assert_eq!(t.unwrap().name, "users");

        let t = s.find_table(&["users", "other_schema.users"]);
        assert_eq!(t.unwrap().name, "users");

        let t = s.find_table(&["other_schema.users", "users"]);
        assert_eq!(t.unwrap().name, "other_schema.users");
    }

    mod transformers_for {
        use super::*;

        fn rule_names(s: &Settings, t: &str) -> Vec<String> {
            s.transformers_for(t)
                .unwrap()
                .iter()
                .map(|(name, _)| name.to_string())
                .collect()
        }

        #[test]
        fn order() {
            let config = r#"
                tables:
                  - name: table1
                    rule_order:
                      - greeting
                      - options
                    rules:
                      options:
                        template:
                          format: "{greeting: \"{{ final.greeting }}\"}"
                      greeting:
                        template:
                          format: "dear {{ final.first_name }} {{ final.last_name }}"
                      first_name:
                        first_name: {}
                      last_name:
                        last_name: {}
                  - name: table2
                    rules:
                      first_name:
                        first_name: {}
                      last_name:
                        last_name: {}
                "#;
            let s = Settings::from_yaml(config, String::new()).unwrap();

            let names = rule_names(&s, "table1");
            assert_eq!(names.len(), 4);
            assert!(names.contains(&"first_name".to_string()));
            assert!(names.contains(&"last_name".to_string()));
            assert_eq!(names[2], "greeting");
            assert_eq!(names[3], "options");

            let names = rule_names(&s, "table2");
            assert_eq!(names.len(), 2);
            assert!(names.contains(&"first_name".to_string()));
            assert!(names.contains(&"last_name".to_string()));

            assert_eq!(s.transformers_for("table3"), None);
        }
    }
}
