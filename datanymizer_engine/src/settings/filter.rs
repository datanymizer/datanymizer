use serde::Deserialize;
use wildmatch::WildMatch;

/// Filter for include or exclude tables
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Eq)]
#[serde(from = "Config")]
pub struct Filter {
    schema: TableList,
    matched_schema: TableList,
    data: TableList,
    matched_data: TableList,
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
        Self::new(full_config.schema, full_config.data)
    }
}

impl Filter {
    pub fn new(schema: TableList, data: TableList) -> Self {
        let matched_schema = TableList::default();
        let matched_data = TableList::default();

        Self {
            schema,
            matched_schema,
            data,
            matched_data,
        }
    }

    pub fn load_tables(&mut self, tables: Vec<String>) {
        self.matched_schema = self.schema.match_with_wildcards(tables.clone());
        self.matched_data = self.data.match_with_wildcards(tables);
    }

    pub fn schema_match_list(&self) -> &TableList {
        &self.matched_schema
    }

    pub fn filter_table(&self, table: &str) -> bool {
        self.matched_schema.filter(table) && self.matched_data.filter(table)
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
pub enum TableList {
    #[serde(rename = "only", alias = "include")]
    Only(Vec<String>),
    #[serde(rename = "except", alias = "exclude")]
    Except(Vec<String>),
}

impl Default for TableList {
    fn default() -> Self {
        Self::Except(Vec::new())
    }
}

impl TableList {
    pub fn tables(&self) -> &Vec<String> {
        match self {
            Self::Only(tables) => tables,
            Self::Except(tables) => tables,
        }
    }

    fn match_with_wildcards(&self, tables: Vec<String>) -> Self {
        let matchers: Vec<_> = self
            .tables()
            .iter()
            .map(|t| WildMatch::new(t.as_str()))
            .collect();
        let matched_tables = tables
            .into_iter()
            .filter(|t| matchers.iter().any(|m| m.matches(t)))
            .collect();

        match self {
            Self::Only(_) => Self::Only(matched_tables),
            Self::Except(_) => Self::Except(matched_tables),
        }
    }

    fn filter(&self, table: &str) -> bool {
        match self {
            Self::Only(tables) => tables.iter().any(|t| t == table),
            Self::Except(tables) => !tables.iter().any(|t| t == table),
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
    #[serde(default)]
    schema: TableList,
    #[serde(default)]
    data: TableList,
}

impl From<TableList> for FullConfig {
    fn from(list: TableList) -> Self {
        Self {
            schema: TableList::default(),
            data: list,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod table_list {
        use super::*;
        use crate::utils::EnumWrapper;

        fn deserialize(config: &str) -> TableList {
            EnumWrapper::parse(config).unwrap()
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
                Filter::new(
                    TableList::default(),
                    TableList::Only(vec![String::from("table1"), String::from("table2")])
                )
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
                Filter::new(
                    TableList::default(),
                    TableList::Except(vec![String::from("table1"), String::from("table2")])
                )
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
                Filter::new(
                    TableList::Only(vec![String::from("table1"), String::from("table2")]),
                    TableList::default(),
                )
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
                Filter::new(
                    TableList::Except(vec![String::from("table1"), String::from("table2")]),
                    TableList::Only(vec![String::from("table1"),]),
                )
            );
        }

        mod load_tables {
            use super::*;

            #[test]
            fn wildcards() {
                let schema =
                    TableList::Except(vec![String::from("public.table1"), String::from("other.*")]);
                let data = TableList::Only(vec![
                    String::from("public.table2"),
                    String::from("public.table1?"),
                    String::from("other.*"),
                ]);
                let tables = vec![
                    String::from("public.table1"),
                    String::from("public.table2"),
                    String::from("public.table3"),
                    String::from("public.table10"),
                    String::from("public.table11"),
                    String::from("other.table1"),
                    String::from("other.table2"),
                    String::from("another.table1"),
                ];

                let mut filter = Filter::new(schema.clone(), data.clone());
                filter.load_tables(tables);

                assert_eq!(
                    filter,
                    Filter {
                        schema,
                        matched_schema: TableList::Except(vec![
                            String::from("public.table1"),
                            String::from("other.table1"),
                            String::from("other.table2"),
                        ]),
                        data,
                        matched_data: TableList::Only(vec![
                            String::from("public.table2"),
                            String::from("public.table10"),
                            String::from("public.table11"),
                            String::from("other.table1"),
                            String::from("other.table2"),
                        ]),
                    }
                )
            }

            #[test]
            fn match_all() {
                let schema = TableList::Only(vec![String::from("*")]);
                let data = TableList::Except(vec![String::from("*")]);
                let tables = vec![
                    String::from("public.table1"),
                    String::from("public.table2"),
                    String::from("other.table1"),
                    String::from("other.table2"),
                ];

                let mut filter = Filter::new(schema.clone(), data.clone());
                filter.load_tables(tables);

                assert_eq!(
                    filter,
                    Filter {
                        schema,
                        matched_schema: TableList::Only(vec![
                            String::from("public.table1"),
                            String::from("public.table2"),
                            String::from("other.table1"),
                            String::from("other.table2"),
                        ]),
                        data,
                        matched_data: TableList::Except(vec![
                            String::from("public.table1"),
                            String::from("public.table2"),
                            String::from("other.table1"),
                            String::from("other.table2"),
                        ]),
                    }
                )
            }
        }

        mod filter_table {
            use super::*;

            #[test]
            fn only_schema() {
                let mut filter = Filter::new(
                    TableList::Except(vec![String::from("table1")]),
                    TableList::default(),
                );
                filter.load_tables(vec![String::from("table1"), String::from("table2")]);
                assert!(!filter.filter_table("table1"));
                assert!(filter.filter_table("table2"));
            }

            #[test]
            fn only_data() {
                let mut filter = Filter::new(
                    TableList::default(),
                    TableList::Only(vec![String::from("table1")]),
                );
                filter.load_tables(vec![String::from("table1"), String::from("table2")]);
                assert!(filter.filter_table("table1"));
                assert!(!filter.filter_table("table2"));
            }

            #[test]
            fn schema_and_data() {
                let mut filter = Filter::new(
                    TableList::Except(vec![String::from("table1")]),
                    TableList::Only(vec![String::from("table1"), String::from("table2")]),
                );
                filter.load_tables(vec![String::from("table1"), String::from("table2")]);
                assert!(!filter.filter_table("table1"));
                assert!(filter.filter_table("table2"));
            }

            #[test]
            fn missed_tables() {
                let mut filter = Filter::new(
                    TableList::default(),
                    TableList::Only(vec![String::from("table1"), String::from("table2")]),
                );
                filter.load_tables(vec![String::from("table1")]);
                assert!(filter.filter_table("table1"));
                assert!(!filter.filter_table("table2"));
            }

            #[test]
            fn wildcards() {
                let mut filter = Filter::new(
                    TableList::Except(vec![String::from("table1*")]),
                    TableList::Only(vec![
                        String::from("table1"),
                        String::from("table2?1"),
                        String::from("table3"),
                    ]),
                );
                filter.load_tables(vec![
                    String::from("table1"),
                    String::from("table10"),
                    String::from("table2"),
                    String::from("table201"),
                    String::from("table3"),
                    String::from("table301"),
                ]);

                assert!(!filter.filter_table("table1"));
                assert!(!filter.filter_table("table10"));
                assert!(!filter.filter_table("table2"));
                assert!(filter.filter_table("table201"));
                assert!(filter.filter_table("table3"));
                assert!(!filter.filter_table("table301"));
            }
        }
    }
}
