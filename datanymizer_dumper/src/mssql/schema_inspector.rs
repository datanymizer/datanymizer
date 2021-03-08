use super::{column::MsSqlColumn, dumper::MsSqlDumper, table::MsSqlTable, MsSqlType};
use crate::{Dumper, SchemaInspector};
use anyhow::Result;

#[derive(Clone)]
pub struct MsSqlSchemaInspector;

impl SchemaInspector for MsSqlSchemaInspector {
    type Type = MsSqlType;
    type Dumper = MsSqlDumper;
    type Table = MsSqlTable;
    type Column = MsSqlColumn;

    fn get_tables(
        &self,
        _connection: &mut <Self::Dumper as Dumper>::Connection,
    ) -> Result<Vec<Self::Table>> {
        Ok(vec![])
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
        _connection: &mut <Self::Dumper as Dumper>::Connection,
        _table: &Self::Table,
    ) -> Result<Vec<Self::Column>> {
        Ok(vec![])
    }
}
