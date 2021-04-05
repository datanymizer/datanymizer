use super::{column::PgColumn, dumper::PgDumper, row::PgRow};
use crate::Table;
use datanymizer_engine::Table as TableCfg;
use postgres::{types::Type, Row as PostgresRow};
use std::{
    collections::HashMap,
    hash::{Hash, Hasher},
};

#[derive(Debug, Clone, Eq)]
pub struct PgTable {
    pub tablename: String,
    pub schemaname: Option<String>,
    pub columns: Vec<PgColumn>,
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
    type Dumper = PgDumper;
    type Column = PgColumn;
    type Row = PgRow<Self>;

    // Returns table name
    fn get_name(&self) -> String {
        self.tablename.clone()
    }

    // Returns table name with the schema or other prefix, based on database type
    fn get_full_name(&self) -> String {
        let mut full_name = String::from("");
        if let Some(schema) = self.schemaname.clone() {
            full_name.push_str(&schema);
            full_name.push('.');
        }
        full_name.push_str(&self.tablename);
        full_name
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
    pub fn new(tablename: String, schemaname: Option<String>) -> Self {
        Self {
            tablename,
            schemaname,
            columns: vec![],
            column_indexes: HashMap::new(),
            size: 0,
        }
    }

    pub fn transformed_query_to(&self, cfg: Option<&TableCfg>) -> Option<String> {
        cfg.map(|c| {
            c.query.as_ref().map_or_else(
                || self.default_query(),
                |q| self.query_with_select(&q.dump_condition, q.limit),
            )
        })
    }

    pub fn untransformed_query_to(&self, cfg: Option<&TableCfg>) -> Option<String> {
        if cfg.is_none() {
            Some(self.default_query())
        } else {
            None
        }
    }

    pub fn count_of_query_to(&self, cfg: Option<&TableCfg>) -> u64 {
        let number = self.get_size() as u64;

        if let Some(c) = cfg {
            if let Some(q) = &c.query {
                if let Some(limit) = q.limit {
                    if number > limit as u64 {
                        return limit as u64;
                    }
                }
            }
        }

        number
    }

    pub fn query_from(&self) -> String {
        format!(
            "COPY {}({}) FROM STDIN;",
            self.get_full_name(),
            self.quoted_columns().join(", "),
        )
    }

    fn default_query(&self) -> String {
        format!(
            "COPY {}({}) TO STDOUT",
            self.get_full_name(),
            self.quoted_columns().join(", ")
        )
    }

    fn query_with_select(&self, cs: &Option<String>, limit: Option<usize>) -> String {
        format!(
            "COPY (SELECT * FROM {}{}{}) TO STDOUT",
            self.get_full_name(),
            Self::sql_conditions(cs),
            Self::sql_limit(limit),
        )
    }

    fn sql_conditions(cs: &Option<String>) -> String {
        cs.as_ref().map_or(String::new(), |conditions| {
            format!(" WHERE ({})", conditions)
        })
    }

    fn sql_limit(limit: Option<usize>) -> String {
        limit.map_or(String::new(), |limit| format!(" LIMIT {}", limit))
    }

    fn quoted_columns(&self) -> Vec<String> {
        self.get_columns_names()
            .into_iter()
            .map(|x| format!("\"{}\"", x))
            .collect()
    }

    pub fn set_columns(&mut self, columns: Vec<PgColumn>) {
        let mut map: HashMap<String, usize> = HashMap::with_capacity(columns.len());
        for column in &columns {
            map.insert(column.name.clone(), (column.position - 1) as usize);
        }

        self.column_indexes = map;
        self.columns = columns;
    }
}

impl From<PostgresRow> for PgTable {
    fn from(row: PostgresRow) -> Self {
        Self::new(row.get("tablename"), row.try_get("schemaname").ok())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{postgres::column::PgColumn, Table};

    #[test]
    fn table_full_name() {
        let table = PgTable::new(String::from("name"), Some(String::from("public")));
        assert_eq!(table.get_name(), String::from("name"));
        assert_eq!(table.get_full_name(), String::from("public.name"));
    }

    #[test]
    fn table_without_scheme() {
        let table = PgTable::new(String::from("name"), None);
        assert_eq!(table.get_name(), String::from("name"));
        assert_eq!(table.get_full_name(), String::from("name"));
    }

    #[test]
    fn set_columns() {
        let mut table = PgTable::new(String::from("name"), None);

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
            position: 3,
            name: String::from("col3"),
            data_type: String::new(),
            inner_type: Some(0),
        };

        table.set_columns(vec![col1.clone(), col2.clone(), col3.clone()]);

        assert_eq!(table.columns, vec![col1, col2, col3]);

        assert_eq!(table.column_indexes.len(), 3);
        assert_eq!(table.column_indexes["col1"], 0);
        assert_eq!(table.column_indexes["col2"], 1);
        assert_eq!(table.column_indexes["col3"], 2);
    }

    mod query_to {
        use super::*;
        use datanymizer_engine::Query;

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
            let mut table = PgTable::new(table_name(), None);
            table.set_columns(columns());
            table.size = 1000;
            table
        }

        fn cfg(query: Option<Query>) -> TableCfg {
            TableCfg {
                name: table_name(),
                rules: HashMap::new(),
                rule_order: None,
                query,
            }
        }

        #[test]
        fn no_table() {
            assert_eq!(table().transformed_query_to(None), None);
            assert_eq!(
                table().untransformed_query_to(None).unwrap(),
                "COPY some_table(\"col1\", \"col2\") TO STDOUT"
            );
            assert_eq!(table().count_of_query_to(None), 1000);
        }

        #[test]
        fn no_query() {
            let cfg = cfg(None);

            assert_eq!(
                table().transformed_query_to(Some(&cfg)).unwrap(),
                "COPY some_table(\"col1\", \"col2\") TO STDOUT"
            );
            assert_eq!(table().untransformed_query_to(Some(&cfg)), None);
            assert_eq!(table().count_of_query_to(Some(&cfg)), 1000);
        }

        #[test]
        fn only_limit() {
            let cfg = cfg(Some(Query {
                limit: Some(100),
                dump_condition: None,
            }));

            assert_eq!(
                table().transformed_query_to(Some(&cfg)).unwrap(),
                "COPY (SELECT * FROM some_table LIMIT 100) TO STDOUT"
            );
            assert_eq!(table().untransformed_query_to(Some(&cfg)), None);
            assert_eq!(table().count_of_query_to(Some(&cfg)), 100);
        }

        #[test]
        fn dump_condition() {
            let cfg = cfg(Some(Query {
                limit: None,
                dump_condition: Some("col1 = 'value'".to_string()),
            }));

            assert_eq!(
                table().transformed_query_to(Some(&cfg)).unwrap(),
                "COPY (SELECT * FROM some_table WHERE (col1 = 'value')) TO STDOUT"
            );
            assert_eq!(table().untransformed_query_to(Some(&cfg)), None);
            assert_eq!(table().count_of_query_to(Some(&cfg)), 1000);
        }
    }
}
