use crate::mssql::{column::MsSqlColumn, dumper::MsSqlDumper, row::MsSqlRow, MsSqlType};
use crate::Table;
use std::{
    collections::HashMap,
    hash::{Hash, Hasher},
};

#[derive(Debug, Clone, Eq)]
pub struct MsSqlTable {
    pub tablename: String,
    pub schemaname: Option<String>,
    pub columns: Vec<MsSqlColumn>,
    column_indexes: HashMap<String, usize>,
    pub size: i64,
}

impl MsSqlTable {
    pub fn new<T: ToString>(name: T, schema: Option<T>) -> Self {
        Self {
            tablename: name.to_string(),
            schemaname: schema.map(|s| s.to_string()),
            columns: vec![],
            column_indexes: HashMap::new(),
            size: 0,
        }
    }
}

impl PartialEq for MsSqlTable {
    fn eq(&self, other: &MsSqlTable) -> bool {
        self.get_full_name() == other.get_full_name()
    }
}

impl Hash for MsSqlTable {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.tablename.hash(state);
        self.schemaname.hash(state);
    }
}

impl Table<MsSqlType> for MsSqlTable {
    type Dumper = MsSqlDumper;
    type Column = MsSqlColumn;
    type Row = MsSqlRow<Self>;

    fn get_name(&self) -> String {
        self.tablename.clone()
    }

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

#[cfg(test)]
mod tests {}
