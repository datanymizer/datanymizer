use super::{
    column::PgColumn, connector, foreign_key::PgForeignKey, sequence::PgSequence, table::PgTable,
    SchemaInspector,
};
use anyhow::Result;
use postgres::types::Type;

const PG_CATALOG_SCHEMA_QUERY: &str = "SELECT tablename, schemaname
    FROM pg_catalog.pg_tables
    WHERE schemaname != 'pg_catalog' AND schemaname != 'information_schema'";

const TABLE_FK_QUERY: &str = "SELECT tc.table_schema,
        tc.constraint_name,
        tc.table_name,
        kcu.column_name,
        ccu.table_schema AS foreign_table_schema,
        ccu.table_name AS foreign_table_name,
        ccu.column_name AS foreign_column_name
    FROM information_schema.table_constraints AS tc
    JOIN information_schema.key_column_usage AS kcu
    ON tc.constraint_name = kcu.constraint_name AND  tc.table_schema = kcu.table_schema
    JOIN information_schema.constraint_column_usage AS ccu
    ON ccu.constraint_name = tc.constraint_name AND ccu.table_schema = tc.table_schema
    WHERE tc.constraint_type = 'FOREIGN KEY' AND tc.table_schema = $1 AND tc.table_name = $2";

const TABLE_COLUMNS_QUERY: &str = "SELECT cc.column_name, cc.ordinal_position, cc.data_type, pt.oid
    FROM information_schema.columns as cc
    JOIN pg_catalog.pg_namespace as pn
    ON cc.udt_schema = pn.nspname
    JOIN pg_catalog.pg_type as pt
    ON cc.udt_name = pt.typname AND pn.oid = pt.typnamespace
    WHERE cc.table_schema = $1 and cc.table_name = $2
    ORDER BY cc.ordinal_position ASC";

const TABLE_SIZE_QUERY: &str =
    "SELECT
    (pg_catalog.pg_class.reltuples / COALESCE(NULLIF(pg_catalog.pg_class.relpages, 0), 1))::bigint * (
        pg_relation_size(pg_catalog.pg_class.oid)::bigint /
        current_setting('block_size')::bigint
    )::bigint AS len
    FROM pg_catalog.pg_class
    INNER JOIN pg_catalog.pg_namespace ON pg_catalog.pg_class.relnamespace = pg_catalog.pg_namespace.oid
    WHERE pg_catalog.pg_namespace.nspname = $1 AND pg_catalog.pg_class.relname = $2";

const SEQUENCE_QUERY: &str = "SELECT pg_catalog.pg_get_serial_sequence($1, $2)";

#[derive(Clone)]
pub struct PgSchemaInspector;

impl SchemaInspector for PgSchemaInspector {
    type Type = Type;
    type Connection = connector::Connection;
    type Table = PgTable;
    type Column = PgColumn;
    type ForeignKey = PgForeignKey;

    // Get all tables in the database
    fn get_tables(&self, connection: &mut Self::Connection) -> Result<Vec<Self::Table>> {
        let mut counter = 0;
        let items: Vec<Self::Table> = connection
            .client
            .query(PG_CATALOG_SCHEMA_QUERY, &[])?
            .into_iter()
            .map(|row| row.into())
            .map(|mut table| {
                if let Ok(columns) = self.get_columns(connection, &table) {
                    table.set_columns(columns);
                };
                if let Ok(sequences) = self.get_sequences(connection, &table) {
                    table.set_sequences(sequences);
                };
                if let Ok(foreign_keys) = self.get_foreign_keys(connection, &table) {
                    table.set_foreign_keys(foreign_keys);
                };

                match self.get_table_size(connection, &table) {
                    Ok(size) => table.size = size,
                    Err(e) => panic!("ERR: {e}"),
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
        connection: &mut Self::Connection,
        table: &Self::Table,
    ) -> Result<i64> {
        let row = connection
            .client
            .query_one(TABLE_SIZE_QUERY, &[&table.schemaname, &table.tablename])?;
        let size: i64 = row.get("len");
        Ok(size)
    }

    /// Get foreign keys for table
    fn get_foreign_keys(
        &self,
        connection: &mut Self::Connection,
        table: &Self::Table,
    ) -> Result<Vec<Self::ForeignKey>> {
        Ok(connection
            .client
            .query(TABLE_FK_QUERY, &[&table.schemaname, &table.tablename])?
            .into_iter()
            .map(|row| row.into())
            .collect())
    }

    /// Get columns for table
    fn get_columns(
        &self,
        connection: &mut Self::Connection,
        table: &Self::Table,
    ) -> Result<Vec<Self::Column>> {
        let items: Vec<Self::Column> = connection
            .client
            .query(TABLE_COLUMNS_QUERY, &[&table.schemaname, &table.tablename])?
            .into_iter()
            .map(|row| row.into())
            .collect();
        Ok(items)
    }
}

impl PgSchemaInspector {
    pub fn get_sequences(
        &self,
        connection: &mut <Self as SchemaInspector>::Connection,
        table: &<Self as SchemaInspector>::Table,
    ) -> Result<Vec<PgSequence>> {
        let mut sequences = vec![];
        for col in table.columns.iter() {
            let full_name: Option<String> = connection
                .client
                .query_one(SEQUENCE_QUERY, &[&table.quoted_full_name(), &col.name])?
                .get(0);
            if let Some(full_name) = full_name {
                sequences.push(PgSequence { full_name });
            }
        }

        Ok(sequences)
    }
}
