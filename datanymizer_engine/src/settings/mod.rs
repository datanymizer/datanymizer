mod filter;
mod table;
mod templates;

use crate::{
    transformer::{TransformerDefaults, TransformerInitContext},
    transformers::Transformers,
    Transformer,
};
use anyhow::Result;
use config::{Config, ConfigError, File, FileFormat};
use serde::Deserialize;
use serde_json::Value as JsonValue;
use std::collections::HashMap;

pub use filter::{Filter, TableList};
pub use table::{Query, Table};
pub use templates::TemplatesCollection;

pub type Tables = Vec<Table>;

type TransformList = Vec<(String, Transformers)>;

#[derive(Debug, Default, Deserialize, Clone)]
pub struct Settings {
    /// Tables list with transformation rules
    #[serde(default)]
    pub tables: Tables,

    /// Table order. All tables not listed are dumping at the beginning
    #[serde(default)]
    pub table_order: Vec<String>,

    /// Default transformers configuration
    #[serde(default)]
    pub default: TransformerDefaults,

    #[serde(default)]
    pub filter: Filter,

    /// Global values. Visible in any template.
    /// They may be shadowed by template variables.
    pub globals: Option<HashMap<String, JsonValue>>,

    pub templates: Option<TemplatesCollection>,

    #[serde(skip)]
    transform_map: Option<HashMap<String, TransformList>>,
}

impl Settings {
    pub fn new(path: String) -> Result<Self, ConfigError> {
        Self::from_source(File::with_name(&path))
    }

    pub fn from_yaml(config: &str) -> Result<Self, ConfigError> {
        Self::from_source(File::from_str(config, FileFormat::Yaml))
    }

    fn from_source<S: 'static>(source: S) -> Result<Self, ConfigError>
    where
        S: config::Source + Send + Sync,
    {
        let mut s = Config::new();
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

    pub fn find_table<T: AsRef<str>>(&self, names: &[T]) -> Option<&Table> {
        for name in names {
            let table = self.get_table(name.as_ref());
            if table.is_some() {
                return table;
            }
        }
        None
    }

    fn preprocess(&mut self) {
        let mut init_ctx = TransformerInitContext::from_defaults(self.default.clone());

        // Assign extend templates to context
        if let Some(collection) = &self.templates {
            init_ctx.template_collection = collection.clone();
        }

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

        let s = Settings::from_yaml(config).unwrap();
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
        let s = Settings::from_yaml(config).unwrap();

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
            let s = Settings::from_yaml(config).unwrap();

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

    mod templates_for {
        use super::*;

        fn get_raw_templates(s: &Settings) -> Vec<String> {
            s.templates
                .clone()
                .unwrap()
                .raw
                .unwrap()
                .iter()
                .map(|(name, _)| name.to_string())
                .collect()
        }

        fn get_files_templates(s: &Settings) -> Vec<String> {
            s.templates.clone().unwrap().files.unwrap()
        }

        #[test]
        fn read_templates() {
            let config = r#"
                tables: []
                templates:
                  raw:
                    template1: "template1"
                    template2: |
                      template2-line-1
                      template2-line-2
                  files:
                    - ./templates/path1
                    - ./templates/path2
                "#;
            let s = Settings::from_yaml(config).unwrap();

            assert_eq!(get_raw_templates(&s).len(), 2);
            assert_eq!(get_files_templates(&s).len(), 2);
        }
    }
}
