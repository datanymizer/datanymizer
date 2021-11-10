use super::{column::PgColumn, row::PgRow, sequence::PgSequence};
use crate::Table;
use anyhow::{anyhow, Result};
use datanymizer_engine::{Query as QueryCfg, Table as TableCfg};
use postgres::{types::Type, Row as PostgresRow};
use std::{
    collections::HashMap,
    hash::{Hash, Hasher},
};

#[derive(Debug, Clone, Eq)]
pub struct PgTable {
    pub tablename: String,
    pub schemaname: String,
    pub columns: Vec<PgColumn>,
    pub sequences: Vec<PgSequence>,
    column_indexes: HashMap<String, usize>,
    pub size: i64,
}

impl PartialEq for PgTable {
    fn eq(&self, other: &PgTable) -> bool {
        self.get_full_name() == other.get_full_name()
    }
}

impl Hash for PgTable {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.tablename.hash(state);
        self.schemaname.hash(state);
    }
}

impl Table<Type> for PgTable {
    type Column = PgColumn;
    type Row = PgRow<Self>;

    // Returns table name
    fn get_name(&self) -> String {
        self.tablename.clone()
    }

    // Returns table name with the schema or other prefix, based on database type
    fn get_full_name(&self) -> String {
        format!("{}.{}", self.schemaname, self.tablename)
    }

    fn get_names(&self) -> Vec<String> {
        vec![self.get_full_name(), self.get_name()]
    }

    fn get_columns(&self) -> Vec<Self::Column> {
        self.columns.clone()
    }

    fn get_columns_names(&self) -> Vec<String> {
        self.get_columns().into_iter().map(|c| c.name).collect()
    }

    fn get_size(&self) -> i64 {
        self.size
    }

    fn get_column_indexes(&self) -> &HashMap<String, usize> {
        &self.column_indexes
    }
}

impl PgTable {
    pub fn new(tablename: String, schemaname: String) -> Self {
        Self {
            tablename,
            schemaname,
            columns: vec![],
            sequences: vec![],
            column_indexes: HashMap::new(),
            size: 0,
        }
    }

    pub fn quote_table_name(name: &str) -> Result<String> {
        let parts: Vec<_> = name.split('.').collect();
        match parts.len() {
            1 => Ok(format!(r#""{}""#, name)),
            2 => Ok(format!(r#""{}"."{}""#, parts[0], parts[1])),
            _ => Err(anyhow!("Invalid table name {}", name)),
        }
    }

    pub fn quoted_full_name(&self) -> String {
        format!(r#""{}"."{}""#, self.schemaname, self.tablename)
    }

    pub fn set_columns(&mut self, columns: Vec<PgColumn>) {
        let mut map: HashMap<String, usize> = HashMap::with_capacity(columns.len());
        let mut column_refs: Vec<_> = columns.iter().collect();
        column_refs.sort_by_key(|c| c.position);
        for (i, column) in column_refs.iter().enumerate() {
            map.insert(column.name.clone(), i);
        }

        self.column_indexes = map;
        self.columns = columns;
    }

    pub fn set_sequences(&mut self, sequences: Vec<PgSequence>) {
        self.sequences = sequences;
    }

    pub fn transformed_query_to(
        &self,
        cfg: Option<&TableCfg>,
        already_dumped: u64,
    ) -> Option<String> {
        cfg.and_then(|c| match &c.query {
            Some(q) => self.query_unless_already_dumped(q, |s| format!("({})", s), already_dumped),
            None => Some(self.default_query()),
        })
    }

    pub fn untransformed_query_to(
        &self,
        cfg: Option<&TableCfg>,
        already_dumped: u64,
    ) -> Option<String> {
        match cfg {
            Some(c) => c.query.as_ref().and_then(|q| {
                if q.transform_condition.is_some() {
                    self.query_unless_already_dumped(q, |s| format!("NOT ({})", s), already_dumped)
                } else {
                    None
                }
            }),
            None => Some(self.default_query()),
        }
    }

    pub fn count_of_query_to(&self, cfg: Option<&TableCfg>) -> u64 {
        let number = self.get_size() as u64;

        cfg.and_then(|c| c.query.as_ref())
            .and_then(|q| q.limit)
            .and_then(|limit| {
                if number > limit as u64 {
                    Some(limit as u64)
                } else {
                    None
                }
            })
            .unwrap_or(number)
    }

    pub fn query_from(&self) -> String {
        format!(
            "COPY {}({}) FROM STDIN;",
            self.quoted_full_name(),
            self.quoted_columns().join(", "),
        )
    }

    fn query_unless_already_dumped(
        &self,
        q: &QueryCfg,
        tr_fmt: fn(s: &String) -> String,
        already_dumped: u64,
    ) -> Option<String> {
        if q.limit
            .map_or(false, |limit| limit as u64 <= already_dumped)
        {
            return None;
        }

        Some(self.query_with_select(
            vec![
                q.dump_condition.as_ref().map(|c| format!("({})", c)),
                q.transform_condition.as_ref().map(tr_fmt),
            ],
            q.limit.map(|limit| limit as u64 - already_dumped),
        ))
    }

    fn default_query(&self) -> String {
        format!(
            "COPY {}({}) TO STDOUT",
            self.quoted_full_name(),
            self.quoted_columns().join(", ")
        )
    }

    fn query_with_select(&self, cs: Vec<Option<String>>, limit: Option<u64>) -> String {
        format!(
            "COPY (SELECT * FROM {}{}{}) TO STDOUT",
            self.quoted_full_name(),
            Self::sql_conditions(cs),
            Self::sql_limit(limit),
        )
    }

    fn sql_conditions(cs: Vec<Option<String>>) -> String {
        let conditions: Vec<String> = cs.into_iter().flatten().collect();
        if conditions.is_empty() {
            String::new()
        } else {
            format!(" WHERE {}", conditions.join(" AND "))
        }
    }

    fn sql_limit(limit: Option<u64>) -> String {
        limit.map_or(String::new(), |limit| format!(" LIMIT {}", limit))
    }

    fn quoted_columns(&self) -> Vec<String> {
        self.get_columns_names()
            .into_iter()
            .map(|x| format!("\"{}\"", x))
            .collect()
    }
}

impl From<PostgresRow> for PgTable {
    fn from(row: PostgresRow) -> Self {
        Self::new(row.get("tablename"), row.get("schemaname"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{postgres::column::PgColumn, Table};

    #[test]
    fn table_full_name() {
        let table = PgTable::new(String::from("name"), String::from("public"));
        assert_eq!(table.get_name(), String::from("name"));
        assert_eq!(table.get_full_name(), String::from("public.name"));
    }

    #[test]
    fn quote_table_name() {
        let name = PgTable::quote_table_name("table").unwrap();
        assert_eq!(name, "\"table\"");

        let name = PgTable::quote_table_name("public.table").unwrap();
        assert_eq!(name, "\"public\".\"table\"");

        let name = PgTable::quote_table_name("public.name.");
        assert!(name.is_err());
    }

    #[test]
    fn quoted_full_name() {
        let table = PgTable::new(String::from("name"), String::from("public2"));
        assert_eq!(table.quoted_full_name(), r#""public2"."name""#)
    }

    #[test]
    fn set_columns() {
        let mut table = PgTable::new(String::from("name"), String::from("public"));

        let col1 = PgColumn {
            position: 1,
            name: String::from("col1"),
            data_type: String::new(),
            inner_type: Some(0),
        };
        let col2 = PgColumn {
            position: 2,
            name: String::from("col2"),
            data_type: String::new(),
            inner_type: Some(0),
        };
        let col3 = PgColumn {
            // Column positions in Postgres are not always in sequence
            // (e.g., when we have dropped some column).
            position: 4,
            name: String::from("col4"),
            data_type: String::new(),
            inner_type: Some(0),
        };

        table.set_columns(vec![col1.clone(), col2.clone(), col3.clone()]);

        assert_eq!(table.columns, vec![col1, col2, col3]);

        assert_eq!(table.column_indexes.len(), 3);
        assert_eq!(table.column_indexes["col1"], 0);
        assert_eq!(table.column_indexes["col2"], 1);
        assert_eq!(table.column_indexes["col4"], 2);
    }

    mod query_to {
        use super::*;

        fn table_name() -> String {
            "some_table".to_string()
        }

        fn columns() -> Vec<PgColumn> {
            let col1 = PgColumn {
                position: 1,
                name: String::from("col1"),
                data_type: String::new(),
                inner_type: Some(0),
            };
            let col2 = PgColumn {
                position: 2,
                name: String::from("col2"),
                data_type: String::new(),
                inner_type: Some(0),
            };
            vec![col1, col2]
        }

        fn table() -> PgTable {
            let mut table = PgTable::new(table_name(), String::from("public"));
            table.set_columns(columns());
            table.size = 1000;
            table
        }

        fn cfg(query: Option<QueryCfg>) -> TableCfg {
            TableCfg {
                name: table_name(),
                rules: HashMap::new(),
                rule_order: None,
                query,
            }
        }

        #[test]
        fn no_table() {
            assert_eq!(table().transformed_query_to(None, 0), None);
            assert_eq!(
                table().untransformed_query_to(None, 0).unwrap(),
                "COPY \"public\".\"some_table\"(\"col1\", \"col2\") TO STDOUT"
            );
            assert_eq!(table().count_of_query_to(None), 1000);
        }

        #[test]
        fn no_query() {
            let cfg = cfg(None);

            assert_eq!(
                table().transformed_query_to(Some(&cfg), 0).unwrap(),
                "COPY \"public\".\"some_table\"(\"col1\", \"col2\") TO STDOUT"
            );
            assert_eq!(table().untransformed_query_to(Some(&cfg), 0), None);
            assert_eq!(table().count_of_query_to(Some(&cfg)), 1000);
        }

        #[test]
        fn only_limit() {
            let cfg = cfg(Some(QueryCfg {
                limit: Some(100),
                dump_condition: None,
                transform_condition: None,
            }));

            assert_eq!(
                table().transformed_query_to(Some(&cfg), 0).unwrap(),
                "COPY (SELECT * FROM \"public\".\"some_table\" LIMIT 100) TO STDOUT"
            );
            assert_eq!(table().untransformed_query_to(Some(&cfg), 0), None);
            assert_eq!(table().count_of_query_to(Some(&cfg)), 100);
        }

        #[test]
        fn only_dump_condition() {
            let cfg = cfg(Some(QueryCfg {
                limit: None,
                dump_condition: Some("col1 = 'value'".to_string()),
                transform_condition: None,
            }));

            assert_eq!(
                table().transformed_query_to(Some(&cfg), 0).unwrap(),
                "COPY (SELECT * FROM \"public\".\"some_table\" WHERE (col1 = 'value')) TO STDOUT"
            );
            assert_eq!(table().untransformed_query_to(Some(&cfg), 0), None);
            assert_eq!(table().count_of_query_to(Some(&cfg)), 1000);
        }

        #[test]
        fn only_transform_condition() {
            let cfg = cfg(Some(QueryCfg {
                limit: None,
                dump_condition: None,
                transform_condition: Some("col1 = 'value'".to_string()),
            }));

            assert_eq!(
                table().transformed_query_to(Some(&cfg), 0).unwrap(),
                "COPY (SELECT * FROM \"public\".\"some_table\" WHERE (col1 = 'value')) TO STDOUT"
            );
            assert_eq!(
                table().untransformed_query_to(Some(&cfg), 0).unwrap(),
                "COPY (SELECT * FROM \"public\".\"some_table\" WHERE NOT (col1 = 'value')) TO STDOUT"
            );
            assert_eq!(table().count_of_query_to(Some(&cfg)), 1000);
        }

        #[test]
        fn all_query_params() {
            let cfg = cfg(Some(QueryCfg {
                limit: Some(500),
                dump_condition: Some("col1 = 'value'".to_string()),
                transform_condition: Some("col2 <> 'other_value'".to_string()),
            }));

            assert_eq!(
                table().transformed_query_to(Some(&cfg), 0).unwrap(),
                "COPY (SELECT * FROM \"public\".\"some_table\" \
                WHERE (col1 = 'value') AND (col2 <> 'other_value') LIMIT 500) TO STDOUT"
            );
            assert_eq!(
                table().untransformed_query_to(Some(&cfg), 0).unwrap(),
                "COPY (SELECT * FROM \"public\".\"some_table\" \
                WHERE (col1 = 'value') AND NOT (col2 <> 'other_value') LIMIT 500) TO STDOUT"
            );
            assert_eq!(table().count_of_query_to(Some(&cfg)), 500);
        }

        mod already_dumped {
            use super::*;

            #[test]
            fn no_limit() {
                let cfg = cfg(Some(QueryCfg {
                    limit: None,
                    dump_condition: None,
                    transform_condition: Some("col1 = 'value'".to_string()),
                }));

                assert_eq!(
                    table().transformed_query_to(Some(&cfg), 100).unwrap(),
                    "COPY (SELECT * FROM \"public\".\"some_table\" WHERE (col1 = 'value')) TO STDOUT"
                );
                assert_eq!(
                    table().untransformed_query_to(Some(&cfg), 100).unwrap(),
                    "COPY (SELECT * FROM \"public\".\"some_table\" WHERE NOT (col1 = 'value')) TO STDOUT"
                );
            }

            #[test]
            fn limit_is_greater() {
                let cfg = cfg(Some(QueryCfg {
                    limit: Some(150),
                    dump_condition: None,
                    transform_condition: Some("col1 = 'value'".to_string()),
                }));

                assert_eq!(
                    table().transformed_query_to(Some(&cfg), 100).unwrap(),
                    "COPY (SELECT * FROM \"public\".\"some_table\" WHERE (col1 = 'value') LIMIT 50) TO STDOUT"
                );
                assert_eq!(
                    table().untransformed_query_to(Some(&cfg), 100).unwrap(),
                    "COPY (SELECT * FROM \"public\".\"some_table\" WHERE NOT (col1 = 'value') LIMIT 50) TO STDOUT"
                );
            }

            #[test]
            fn limit_is_equal() {
                let cfg = cfg(Some(QueryCfg {
                    limit: Some(100),
                    dump_condition: None,
                    transform_condition: Some("col1 = 'value'".to_string()),
                }));

                assert_eq!(table().transformed_query_to(Some(&cfg), 100), None);
                assert_eq!(table().untransformed_query_to(Some(&cfg), 100), None);
            }

            #[test]
            fn limit_is_lesser() {
                let cfg = cfg(Some(QueryCfg {
                    limit: Some(99),
                    dump_condition: None,
                    transform_condition: Some("col1 = 'value'".to_string()),
                }));

                assert_eq!(table().transformed_query_to(Some(&cfg), 100), None);
                assert_eq!(table().untransformed_query_to(Some(&cfg), 100), None);
            }
        }
    }
}
