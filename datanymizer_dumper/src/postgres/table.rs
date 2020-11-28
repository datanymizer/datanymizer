use super::{column::PgColumn, dumper::PgDumper, row::PgRow};
use crate::Table;
use postgres::{types::Type, Row as PostgresRow};
use std::hash::{Hash, Hasher};

#[derive(Debug, Clone, Eq)]
pub struct PgTable {
    pub tablename: String,
    pub schemaname: Option<String>,
    pub columns: Vec<PgColumn>,
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
}

impl PgTable {
    pub fn query_to(&self) -> String {
        format!(
            "COPY {}({}) TO STDOUT WITH(FORMAT CSV, NULL '\\N', FORCE_QUOTE *)",
            self.get_full_name(),
            self.quoted_columns().join(","),
        )
    }

    pub fn query_from(&self) -> String {
        format!(
            "COPY {}({}) FROM STDIN WITH CSV NULL '\\N';",
            self.get_full_name(),
            self.quoted_columns().join(","),
        )
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
        Self {
            tablename: row.get("tablename"),
            schemaname: row.try_get("schemaname").ok(),
            columns: vec![],
            size: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::PgTable;
    use crate::Table;

    #[test]
    fn test_table_full_name() {
        let table = PgTable {
            tablename: String::from("name"),
            schemaname: Some(String::from("public")),
            columns: vec![],
            size: 0,
        };
        assert_eq!(table.get_name(), String::from("name"));
        assert_eq!(table.get_full_name(), String::from("public.name"));
    }

    #[test]
    fn test_table_without_scheme() {
        let table = PgTable {
            tablename: String::from("name"),
            schemaname: None,
            columns: vec![],
            size: 0,
        };
        assert_eq!(table.get_name(), String::from("name"));
        assert_eq!(table.get_full_name(), String::from("name"));
    }
}
