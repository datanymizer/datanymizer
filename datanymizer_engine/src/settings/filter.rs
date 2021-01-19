use serde::Deserialize;

/// Filter for include or exclude tables
#[derive(Clone, Debug, Deserialize, PartialEq)]
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
    pub fn filter_schema(&self, table: &str) -> bool {
        Self::filter(&self.schema, table)
    }

    pub fn filter_data(&self, table: &str) -> bool {
        Self::filter(&self.data, table)
    }

    fn filter(list: &Option<TableList>, table: &str) -> bool {
        if let Some(l) = list {
            l.filter(table)
        } else {
            true
        }
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub enum TableList {
    #[serde(rename = "only", alias = "include")]
    Only(Vec<String>),
    #[serde(rename = "except", alias = "exclude")]
    Except(Vec<String>),
}

impl TableList {
    pub fn filter(&self, table: &str) -> bool {
        match self {
            Self::Only(tables) => tables.iter().any(|t| t == table),
            Self::Except(tables) => !tables.iter().any(|t| t == table),
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

#[cfg(test)]
mod tests {
    use super::*;

    mod table_list {
        use super::*;

        fn deserialize(config: &str) -> TableList {
            serde_yaml::from_str(config).unwrap()
        }

        #[test]
        fn only_deserialization() {
            let config = r#"
                only:
                  - table1
                  - table2
                "#;

            assert_eq!(
                deserialize(config),
                TableList::Only(vec![String::from("table1"), String::from("table2")])
            );
        }

        #[test]
        fn include_deserialization() {
            let config = r#"
                include:
                  - table1
                "#;

            assert_eq!(
                deserialize(config),
                TableList::Only(vec![String::from("table1"),])
            );
        }

        #[test]
        fn except_deserialization() {
            let config = r#"
                except:
                  - table1
                "#;

            assert_eq!(
                deserialize(config),
                TableList::Except(vec![String::from("table1"),])
            );
        }

        #[test]
        fn exclude_deserialization() {
            let config = r#"
                exclude:
                  - table3
                  - table4
                "#;

            assert_eq!(
                deserialize(config),
                TableList::Except(vec![String::from("table3"), String::from("table4")])
            );
        }

        #[test]
        fn filter() {
            let list = TableList::Except(vec![String::from("users")]);
            assert!(list.filter(&String::from("posts")));
            assert!(!list.filter(&String::from("users")));

            let list = TableList::Only(vec![String::from("users")]);
            assert!(!list.filter(&String::from("posts")));
            assert!(list.filter(&String::from("users")));
        }

        #[test]
        fn tables() {
            let tables = vec![String::from("table_a"), String::from("table_b")];
            let list = TableList::Except(tables.clone());
            assert_eq!(*list.tables(), tables);

            let list = TableList::Only(tables.clone());
            assert_eq!(*list.tables(), tables);
        }
    }

    mod filter {
        use super::*;

        fn deserialize(config: &str) -> Filter {
            serde_yaml::from_str(config).unwrap()
        }

        #[test]
        fn short_deserialization() {
            let config = r#"
                only:
                  - table1
                  - table2
                "#;
            assert_eq!(
                deserialize(config),
                Filter {
                    schema: None,
                    data: Some(TableList::Only(vec![
                        String::from("table1"),
                        String::from("table2")
                    ])),
                }
            );
        }

        #[test]
        fn full_deserialization_data() {
            let config = r#"
                data:
                  except:
                    - table1
                    - table2
                "#;

            assert_eq!(
                deserialize(config),
                Filter {
                    schema: None,
                    data: Some(TableList::Except(vec![
                        String::from("table1"),
                        String::from("table2")
                    ])),
                }
            );
        }

        #[test]
        fn full_deserialization_schema() {
            let config = r#"
                schema:
                  only:
                    - table1
                    - table2
                "#;

            assert_eq!(
                deserialize(config),
                Filter {
                    schema: Some(TableList::Only(vec![
                        String::from("table1"),
                        String::from("table2")
                    ])),
                    data: None,
                }
            );
        }

        #[test]
        fn full_deserialization_schema_and_data() {
            let config = r#"
                schema:
                  except:
                    - table1
                    - table2
                data:
                  only:
                    - table1
                "#;

            assert_eq!(
                deserialize(config),
                Filter {
                    schema: Some(TableList::Except(vec![
                        String::from("table1"),
                        String::from("table2")
                    ])),
                    data: Some(TableList::Only(vec![String::from("table1"),])),
                }
            );
        }

        #[test]
        fn filter_schema() {
            let filter = Filter {
                schema: Some(TableList::Except(vec![String::from("table1")])),
                data: None,
            };
            assert!(!filter.filter_schema(&String::from("table1")));
            assert!(filter.filter_schema(&String::from("table2")));

            let filter = Filter {
                schema: None,
                data: Some(TableList::Except(vec![String::from("table1")])),
            };
            assert!(filter.filter_schema(&String::from("table1")));
            assert!(filter.filter_schema(&String::from("table2")));
        }

        #[test]
        fn filter_data() {
            let filter = Filter {
                schema: Some(TableList::Except(vec![String::from("table1")])),
                data: None,
            };
            assert!(filter.filter_data(&String::from("table1")));
            assert!(filter.filter_data(&String::from("table2")));

            let filter = Filter {
                schema: None,
                data: Some(TableList::Only(vec![String::from("table1")])),
            };
            assert!(filter.filter_data(&String::from("table1")));
            assert!(!filter.filter_data(&String::from("table2")));
        }
    }
}
