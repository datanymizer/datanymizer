use super::{column::PgColumn, dumper::PgDumper, row::PgRow};
use crate::Table;
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

    // Returns table name with schema or other prefix, based on database type
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

    pub fn query_to(&self) -> String {
        format!(
            "COPY {}({}) TO STDOUT",
            self.get_full_name(),
            self.quoted_columns().join(", "),
        )
    }

    pub fn query_from(&self) -> String {
        format!(
            "COPY {}({}) FROM STDIN;",
            self.get_full_name(),
            self.quoted_columns().join(", "),
        )
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
    use super::PgTable;
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
}
