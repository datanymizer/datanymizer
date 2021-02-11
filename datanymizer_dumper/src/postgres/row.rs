use crate::Table;
use anyhow::Result;
use datanymizer_engine::Engine;
use postgres::types::Type;
use std::{char, collections::HashMap};

#[derive(Debug)]
pub struct PgRow<T>
where
    T: Table<Type>,
{
    table: T,
    source: String,
}

impl<T> PgRow<T>
where
    T: Table<Type>,
{
    pub fn from_string_row(source: String, parent_table: T) -> Self {
        Self {
            source,
            table: parent_table,
        }
    }

    /// Applies the transform engine to every column in the row
    /// Returns a new StringRecord for store in the dump
    pub fn transform(&self, engine: &Engine) -> Result<String> {
        let split_char: char = char::from_u32(0x0009).unwrap();
        let column_names = self.table.get_columns_names();
        let mut value_map = HashMap::with_capacity(column_names.len());
        for (i, v) in self.source.split(split_char).enumerate() {
            value_map.insert(column_names[i].clone(), v.to_string());
        }

        engine.process_row(self.table.get_name(), &mut value_map)?;

        Ok(column_names
            .iter()
            .map(|c| value_map[c].as_str())
            .collect::<Vec<&str>>()
            .join("\t"))
    }
}
