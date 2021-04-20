use super::TransformList;
use crate::Transformers;
use serde::Deserialize;
use std::collections::HashMap;

type Rules = HashMap<String, Transformers>;

#[derive(Debug, Deserialize, Clone)]
pub struct Query {
    /// SQL limit
    pub limit: Option<usize>,
    /// SQL condition (WHERE) for dumping
    pub dump_condition: Option<String>,
    /// SQL condition (WHERE) for transforming (anonymizing)
    pub transform_condition: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Table {
    /// Table name
    pub name: String,
    /// Rule set for columns
    pub rules: Rules,
    /// Order of applying rules. All rules not listed are placed at the beginning
    pub rule_order: Option<Vec<String>>,
    /// Limit and conditions for the dumping query
    pub query: Option<Query>,
}

impl Table {
    pub fn transform_list(&self) -> TransformList {
        let explicit_rule_order = self.rule_order.clone().unwrap_or_default();
        let mut transform_list: TransformList = self
            .rules
            .iter()
            .map(|(key, ts)| (key.clone(), ts.clone()))
            .collect();
        transform_list
            .sort_by_cached_key(|(key, _)| explicit_rule_order.iter().position(|i| i == key));

        transform_list
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod transform_list {
        use super::*;

        fn rule_names(t: &Table) -> Vec<String> {
            t.transform_list()
                .iter()
                .map(|(name, _)| name.to_string())
                .collect()
        }

        #[test]
        fn order() {
            let config = r#"
                name: table1
                rule_order:
                  - first_name
                  - greeting
                  - options
                rules:
                  options:
                    template:
                      format: "{greeting: \"{{ final.greeting }}\"}"
                  greeting:
                    template:
                      format: "dear {{ final.first_name }} {{ final.middle_name }} {{ final.last_name }}"
                  first_name:
                    first_name: {}
                  middle_name:
                    last_name: {}
                  last_name:
                    last_name: {}
                "#;
            let t: Table = serde_yaml::from_str(config).unwrap();

            let names = rule_names(&t);

            assert_eq!(names.len(), 5);
            assert!(names.contains(&"middle_name".to_string()));
            assert!(names.contains(&"last_name".to_string()));
            assert_eq!(names[2], "first_name");
            assert_eq!(names[3], "greeting");
            assert_eq!(names[4], "options");
        }
    }
}
