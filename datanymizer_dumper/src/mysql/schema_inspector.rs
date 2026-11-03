use super::{Connection, MyColumn, MyForeignKey, MyTable};
use crate::SchemaInspector;
use anyhow::Result;
use mysql::{consts::ColumnType, prelude::Queryable};

const TABLES_QUERY: &str = "SELECT TABLE_NAME, TABLE_SCHEMA
    FROM information_schema.TABLES
    WHERE TABLE_SCHEMA = DATABASE() AND TABLE_TYPE = 'BASE TABLE'";

const TABLE_SIZE_QUERY: &str = "SELECT COALESCE(TABLE_ROWS, 0)
    FROM information_schema.TABLES
    WHERE TABLE_SCHEMA = ? AND TABLE_NAME = ?";

const TABLE_FK_QUERY: &str = "SELECT CONSTRAINT_NAME, COLUMN_NAME,
        REFERENCED_TABLE_SCHEMA, REFERENCED_TABLE_NAME, REFERENCED_COLUMN_NAME
    FROM information_schema.KEY_COLUMN_USAGE
    WHERE TABLE_SCHEMA = ? AND TABLE_NAME = ? AND REFERENCED_TABLE_NAME IS NOT NULL";

// Generated columns can't be inserted, so they are skipped
// (MySQL has '' and MariaDB has NULL in GENERATION_EXPRESSION for regular columns)
const TABLE_COLUMNS_QUERY: &str = "SELECT COLUMN_NAME, ORDINAL_POSITION, DATA_TYPE
    FROM information_schema.COLUMNS
    WHERE TABLE_SCHEMA = ? AND TABLE_NAME = ? AND COALESCE(GENERATION_EXPRESSION, '') = ''
    ORDER BY ORDINAL_POSITION";

#[derive(Clone)]
pub struct MySchemaInspector;

impl SchemaInspector for MySchemaInspector {
    type Type = ColumnType;
    type Connection = Connection;
    type Table = MyTable;
    type Column = MyColumn;
    type ForeignKey = MyForeignKey;

    fn get_tables(&self, connection: &mut Self::Connection) -> Result<Vec<Self::Table>> {
        let names: Vec<(String, String)> = connection.conn.query(TABLES_QUERY)?;
        names
            .into_iter()
            .map(|(name, schema)| {
                let mut table = MyTable::new(name, schema);
                table.set_columns(self.get_columns(connection, &table)?);
                table.foreign_keys = self.get_foreign_keys(connection, &table)?;
                table.size = self.get_table_size(connection, &table)?;
                Ok(table)
            })
            .collect()
    }

    fn get_table_size(
        &self,
        connection: &mut Self::Connection,
        table: &Self::Table,
    ) -> Result<i64> {
        let size: Option<u64> = connection
            .conn
            .exec_first(TABLE_SIZE_QUERY, (&table.schemaname, &table.tablename))?;
        Ok(size.unwrap_or_default() as i64)
    }

    fn get_foreign_keys(
        &self,
        connection: &mut Self::Connection,
        table: &Self::Table,
    ) -> Result<Vec<Self::ForeignKey>> {
        Ok(connection.conn.exec_map(
            TABLE_FK_QUERY,
            (&table.schemaname, &table.tablename),
            |(
                constraint_name,
                column_name,
                foreign_table_schema,
                foreign_table_name,
                foreign_column_name,
            )| {
                MyForeignKey {
                    constraint_name,
                    column_name,
                    foreign_table_schema,
                    foreign_table_name,
                    foreign_column_name,
                }
            },
        )?)
    }

    fn get_columns(
        &self,
        connection: &mut Self::Connection,
        table: &Self::Table,
    ) -> Result<Vec<Self::Column>> {
        Ok(connection.conn.exec_map(
            TABLE_COLUMNS_QUERY,
            (&table.schemaname, &table.tablename),
            |(name, position, data_type): (String, u64, String)| MyColumn {
                position: position as usize,
                name,
                data_type,
            },
        )?)
    }
}
