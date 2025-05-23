use super::{column::MysqlColumn, connector::Connection, table::MysqlTable, SchemaInspector};

use crate::Table;
use anyhow::Result;
use sqlx::{mysql::MySqlRow, Row};

const TABLE_LIST_QUERY: &str = "SHOW FULL TABLES WHERE Table_Type = 'BASE TABLE'";
const COLUMN_LIST_QUERY: &str =
    "SELECT COLUMN_NAME, ORDINAL_POSITION, DATA_TYPE FROM information_schema.columns \
     WHERE TABLE_SCHEMA = ? AND TABLE_NAME = ?";
const TABLE_SIZE_QUERY: &str = "SELECT TABLE_ROWS FROM information_schema.tables \
                                WHERE TABLE_SCHEMA = ? AND TABLE_NAME = ?";

#[derive(Clone)]
pub struct MysqlSchemaInspector {}

impl SchemaInspector for MysqlSchemaInspector {
    type Type = String;
    type Connection = Connection;
    type Table = MysqlTable;
    type Column = MysqlColumn;
    type ForeignKey = ();

    fn get_tables(&self, connection: &mut Self::Connection) -> Result<Vec<Self::Table>> {
        let mut table_names: Vec<String> = connection.rt.block_on(
            sqlx::query(TABLE_LIST_QUERY)
                .map(|row: MySqlRow| row.get(0))
                .fetch_all(&mut connection.conn),
        )?;
        table_names.sort();
        let mut tables: Vec<MysqlTable> = table_names.into_iter().map(MysqlTable::new).collect();
        for table in &mut tables {
            table.set_columns(self.get_columns(connection, table)?);
            match self.get_table_size(connection, table) {
                Ok(size) => table.size = size as usize,
                Err(e) => panic!("ERR: {}", e),
            }
        }
        Ok(tables)
    }

    fn get_table_size(
        &self,
        connection: &mut Self::Connection,
        table: &Self::Table,
    ) -> Result<i64> {
        let size: u64 = connection.rt.block_on(
            sqlx::query(TABLE_SIZE_QUERY)
                .bind(connection.db_name().to_string())
                .bind(table.get_name())
                .map(|row: MySqlRow| row.get(0))
                .fetch_one(&mut connection.conn),
        )?;

        Ok(size as i64)
    }

    fn get_foreign_keys(
        &self,
        _connection: &mut Self::Connection,
        _table: &Self::Table,
    ) -> Result<Vec<Self::ForeignKey>> {
        todo!()
    }

    fn ordered_tables(&self, connection: &mut Self::Connection) -> Result<Vec<(Self::Table, i32)>> {
        self.get_tables(connection)
            .map(|tables| tables.into_iter().map(|tbl| (tbl, 0)).collect())
    }

    fn get_columns(
        &self,
        connection: &mut Self::Connection,
        table: &Self::Table,
    ) -> Result<Vec<Self::Column>> {
        let columns: Vec<MysqlColumn> = connection.rt.block_on(
            sqlx::query(COLUMN_LIST_QUERY)
                .bind(connection.db_name().to_string())
                .bind(table.get_name())
                .map(|row: MySqlRow| {
                    MysqlColumn::new(row.get(0), (row.get::<u64, _>(1) - 1) as usize, row.get(2))
                })
                .fetch_all(&mut connection.conn),
        )?;

        Ok(columns)
    }
}
