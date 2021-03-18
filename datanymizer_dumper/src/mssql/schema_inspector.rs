use super::{column::MsSqlColumn, dumper::MsSqlDumper, table::MsSqlTable, MsSqlType};
use crate::{Dumper, SchemaInspector};
use anyhow::Result;
use async_std::task;

#[derive(Clone)]
pub struct MsSqlSchemaInspector;

impl SchemaInspector for MsSqlSchemaInspector {
    type Type = MsSqlType;
    type Dumper = MsSqlDumper;
    type Table = MsSqlTable;
    type Column = MsSqlColumn;

    fn get_tables(
        &self,
        connection: &mut <Self::Dumper as Dumper>::Connection,
    ) -> Result<Vec<Self::Table>> {
        task::block_on(async {
            let tables = connection
                .simple_query(
                    "SELECT TABLE_NAME FROM INFORMATION_SCHEMA.TABLES WHERE TABLE_TYPE='BASE TABLE'",
                )
                .await?
                .into_first_result()
                .await?
                .iter()
                .map(|row| MsSqlTable::new(row.get::<&str, _>(0).unwrap()))
                .collect();
            Ok(tables)
        })
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
