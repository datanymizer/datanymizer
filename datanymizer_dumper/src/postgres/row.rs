use crate::Table;
use anyhow::Result;
use datanymizer_engine::Engine;
use postgres::types::Type;
use std::char;

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
        let values: Vec<_> = self.source.split(split_char).collect();
        let transformed_values = engine.process_row(
            self.table.get_name(),
            self.table.get_column_indexes(),
            &values,
        )?;

        Ok(transformed_values.join("\t"))
    }
}
