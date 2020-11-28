use crate::{ColumnData, Table};
use anyhow::Result;
use csv::StringRecord;
use datanymizer_engine::{Engine, StringValue};
use postgres::types::Type;

#[derive(Debug)]
pub struct PgRow<T>
where
    T: Table<Type>,
{
    table: T,
    source: StringRecord,
}

impl<T> PgRow<T>
where
    T: Table<Type>,
{
    pub fn from_string_row(source: StringRecord, parent_table: T) -> Self {
        Self {
            source,
            table: parent_table,
        }
    }

    /// Apply transform engine to every colomn in row
    /// Returs new StringRecord for store in dump
    pub fn transform(&self, engine: &Engine) -> Result<StringRecord> {
        let mut result = StringRecord::new();
        for col in self.table.get_columns().iter() {
            let pos = col.position();
            let value = self.source.get(pos as usize).unwrap_or("");
            let mut dto = StringValue {
                table_name: self.table.get_name().clone(),
                field_name: col.name().to_string(),
                value: value.to_string(),
            };
            let _ = engine.process(&mut dto)?;

            dto.update(dto.value.clone());

            result.push_field(&dto.value);
        }
        Ok(result)
    }
}
