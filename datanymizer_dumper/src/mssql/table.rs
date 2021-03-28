use crate::mssql::{column::MsSqlColumn, dumper::MsSqlDumper, row::MsSqlRow, sql_type::MsSqlType};
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

    pub fn set_columns(&mut self, columns: Vec<MsSqlColumn>) {
        let mut map: HashMap<String, usize> = HashMap::with_capacity(columns.len());
        for column in &columns {
            map.insert(column.name.clone(), (column.position - 1) as usize);
        }

        self.column_indexes = map;
        self.columns = columns;
    }

    pub(crate) fn query_from(&self) -> String {
        format!(
            "SELECT {} FROM {}",
            self.columns
                .iter()
                .map(|c| c.expression_for_query_from())
                .collect::<Vec<_>>()
                .join(", "),
            self.full_escaped_name()
        )
    }

    pub(crate) fn identity_insert_on(&self) -> String {
        format!("SET IDENTITY_INSERT {} ON", self.full_escaped_name())
    }

    pub(crate) fn identity_insert_off(&self) -> String {
        format!("SET IDENTITY_INSERT {} OFF", self.full_escaped_name())
    }

    pub(crate) fn insert_statement(&self) -> String {
        format!(
            "INSERT INTO {} ({}) VALUES",
            self.full_escaped_name(),
            self.escaped_columns().join(", ")
        )
    }

    fn full_escaped_name(&self) -> String {
        vec![self.schemaname.clone(), Some(self.tablename.clone())]
            .iter()
            .filter_map(|i| i.as_ref().map(|s| format!("[{}]", s)))
            .collect::<Vec<_>>()
            .join(".")
    }

    fn escaped_columns(&self) -> Vec<String> {
        self.get_columns_names()
            .into_iter()
            .map(|x| format!("[{}]", x))
            .collect()
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
