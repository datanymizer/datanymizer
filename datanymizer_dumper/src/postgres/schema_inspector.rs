use super::{
    column::PgColumn, dumper::PgDumper, foreign_key::ForeignKey, table::PgTable, Dumper,
    SchemaInspector,
};
use crate::Table;
use anyhow::Result;
use postgres::types::Type;
use std::vec::Vec;

const PG_CATALOG_SCHEMA: &str = "SELECT tablename, schemaname
                                 FROM pg_catalog.pg_tables
                                 WHERE schemaname != 'pg_catalog'
                                 AND schemaname != 'information_schema'";

const TABLE_FOREIGN_KEYS: &str = "SELECT
                                    tc.table_schema,
                                    tc.constraint_name,
                                    tc.table_name,
                                    kcu.column_name,
                                    ccu.table_schema AS foreign_table_schema,
                                    ccu.table_name AS foreign_table_name,
                                    ccu.column_name AS foreign_column_name
                                FROM
                                    information_schema.table_constraints AS tc
                                    JOIN information_schema.key_column_usage AS kcu
                                    ON tc.constraint_name = kcu.constraint_name
                                    AND tc.table_schema = kcu.table_schema
                                    JOIN information_schema.constraint_column_usage AS ccu
                                    ON ccu.constraint_name = tc.constraint_name
                                    AND ccu.table_schema = tc.table_schema
                                WHERE tc.constraint_type = 'FOREIGN KEY' AND tc.table_name=$1";

const TABLE_COLUMNS_QUERY: &str = "SELECT cc.column_name, cc.ordinal_position, cc.data_type, pt.oid
                                   FROM information_schema.columns as cc
                                   JOIN pg_catalog.pg_type as pt
                                   ON cc.udt_name = pt.typname
                                   WHERE cc.table_schema=$1 and cc.table_name = $2
                                   ORDER BY cc.ordinal_position ASC";

#[derive(Clone)]
pub struct PgSchemaInspector;

impl SchemaInspector for PgSchemaInspector {
    type Type = Type;
    type Dumper = PgDumper;
    type Table = PgTable;
    type Column = PgColumn;

    // Get all tables in the database
    fn get_tables(
        &self,
        connection: &mut <Self::Dumper as Dumper>::Connection,
    ) -> Result<Vec<Self::Table>> {
        let mut counter = 0;
        let items: Vec<Self::Table> = connection
            .query(PG_CATALOG_SCHEMA, &[])?
            .into_iter()
            .map(|row| row.into())
            .map(|mut table| {
                if let Ok(columns) = self.get_columns(connection, &table) {
                    table.set_columns(columns);
                };

                match self.get_table_size(connection, table.get_name()) {
                    Ok(size) => table.size = size as i64,
                    Err(e) => panic!("ERR: {}", e),
                }

                counter += 1;

                table
            })
            .collect();
        Ok(items)
    }

    /// Get table size
    fn get_table_size(
        &self,
        connection: &mut <Self::Dumper as Dumper>::Connection,
        table_name: String,
    ) -> Result<i64> {
        let query: &str = &format!(
            "SELECT
            (reltuples/COALESCE(NULLIF(relpages, 0), 1))::bigint * (
                pg_relation_size('{table_name}')::bigint /
                (current_setting('block_size')::bigint)
            )::bigint as len
            FROM pg_class where relname = '{table_name}'",
            table_name = table_name
        );

        let row = connection.query_one(query, &[])?;
        let size: i64 = row.get("len");
        Ok(size)
    }

    // Get all dependencies (by FK) for `table` in database
    fn get_dependencies(
        &self,
        connection: &mut <Self::Dumper as Dumper>::Connection,
        table: &Self::Table,
    ) -> Result<Vec<Self::Table>> {
        let fkeys: Vec<ForeignKey> = connection
            .query(TABLE_FOREIGN_KEYS, &[&table.get_name()])?
            .into_iter()
            .map(|row| row.into())
            .collect();

        let tables: Vec<Self::Table> = fkeys
            .into_iter()
            // Table from foreign key
            .map(|fkey| PgTable::new(fkey.foreign_table_name, Some(fkey.foreign_table_schema)))
            // Columns for table
            .map(|mut table| {
                if let Ok(columns) = self.get_columns(connection, &table) {
                    table.set_columns(columns);
                };

                match self.get_table_size(connection, table.get_name()) {
                    Ok(size) => table.size = size as i64,
                    Err(e) => println!("ERR: {}", e),
                }

                table
            })
            .collect();
        Ok(tables)
    }

    /// Get columnst for table
    fn get_columns(
        &self,
        connection: &mut <Self::Dumper as Dumper>::Connection,
        table: &Self::Table,
    ) -> Result<Vec<Self::Column>> {
        let items: Vec<Self::Column> = connection
            .query(TABLE_COLUMNS_QUERY, &[&table.schemaname, &table.tablename])?
            .into_iter()
            .map(|row| row.into())
            .collect();
        Ok(items)
    }
}
