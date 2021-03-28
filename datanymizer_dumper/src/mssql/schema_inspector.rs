use super::{column::MsSqlColumn, dumper::MsSqlDumper, sql_type::MsSqlType, table::MsSqlTable};
use crate::{Dumper, SchemaInspector, Table};
use anyhow::Result;

#[derive(Clone)]
pub struct MsSqlSchemaInspector;

impl SchemaInspector for MsSqlSchemaInspector {
    type Type = MsSqlType;
    type Dumper = MsSqlDumper;
    type Table = MsSqlTable;
    type Column = MsSqlColumn;

    fn get_tables(&self, c: &mut <Self::Dumper as Dumper>::Connection) -> Result<Vec<Self::Table>> {
        let tables = Self::Dumper::query(
            c,
            "SELECT TABLE_NAME, TABLE_SCHEMA FROM INFORMATION_SCHEMA.TABLES \
            WHERE TABLE_TYPE='BASE TABLE'",
        )?
        .iter()
        .map(|row| {
            let name = row.get::<&str, _>(0).expect("table name column is missed");
            let schema = row.get::<&str, _>(1).expect("schema name column is missed");
            let mut table = MsSqlTable::new(name, Some(schema));

            if let Ok(columns) = self.get_columns(c, &table) {
                table.set_columns(columns);
            };

            table
        })
        .collect();
        Ok(tables)
    }

    /// Get table size
    fn get_table_size(
        &self,
        _connection: &mut <Self::Dumper as Dumper>::Connection,
        _table_name: String,
    ) -> Result<i64> {
        Ok(100)
    }

    fn get_dependencies(
        &self,
        _connection: &mut <Self::Dumper as Dumper>::Connection,
        _table: &Self::Table,
    ) -> Result<Vec<Self::Table>> {
        Ok(vec![])
    }

    /// Get columns for table
    fn get_columns(
        &self,
        c: &mut <Self::Dumper as Dumper>::Connection,
        table: &Self::Table,
    ) -> Result<Vec<Self::Column>> {
        let columns = Self::Dumper::query(
            c,
            format!(
                "SELECT COLUMN_NAME, ORDINAL_POSITION, DATA_TYPE FROM INFORMATION_SCHEMA.COLUMNS \
                WHERE TABLE_NAME=N'{}' ORDER BY ORDINAL_POSITION",
                table.get_name()
            )
            .as_str(),
        )?
        .iter()
        .map(Self::Column::from)
        .collect();

        Ok(columns)
    }
}
