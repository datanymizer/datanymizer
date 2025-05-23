use super::column::MysqlColumn;
use crate::{ColumnData, Table};
use std::{
    collections::HashMap,
    hash::{Hash, Hasher},
};

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct MysqlTable {
    pub name: String,
    pub columns: Vec<MysqlColumn>,
    pub size: usize,
    column_indexes: HashMap<String, usize>,
}

impl Hash for MysqlTable {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state);
    }
}

impl Table<String> for MysqlTable {
    type Column = MysqlColumn;
    type Row = ();

    fn get_name(&self) -> String {
        self.name.clone()
    }

    fn get_full_name(&self) -> String {
        self.name.clone()
    }

    fn get_columns(&self) -> Vec<Self::Column> {
        self.columns.clone()
    }

    fn get_columns_names(&self) -> Vec<String> {
        self.get_columns()
            .into_iter()
            .map(|c| c.name().to_string())
            .collect()
    }

    fn get_size(&self) -> i64 {
        self.size as i64
    }

    fn get_column_indexes(&self) -> &HashMap<String, usize> {
        &self.column_indexes
    }

    fn get_dep_table_names(&self) -> Vec<String> {
        todo!()
    }

    fn query_with_select(&self, cs: Vec<Option<String>>, limit: Option<u64>) -> String {
        format!(
            "SELECT * FROM `{}` {}{}",
            self.get_name(),
            Self::sql_conditions(cs),
            Self::sql_limit(limit),
        )
    }

    fn default_query(&self) -> String {
        format!("SELECT * FROM `{}`", self.get_name())
    }
}

impl MysqlTable {
    pub fn new(name: String) -> Self {
        Self {
            name,
            columns: vec![],
            size: 0,
            column_indexes: HashMap::new(),
        }
    }

    pub fn set_columns(&mut self, columns: Vec<MysqlColumn>) {
        let mut map: HashMap<String, usize> = HashMap::with_capacity(columns.len());
        let column_refs: Vec<_> = columns.iter().collect();
        for (i, column) in column_refs.iter().enumerate() {
            map.insert(column.name().to_string(), i);
        }

        self.column_indexes = map;
        self.columns = columns;
    }
}
