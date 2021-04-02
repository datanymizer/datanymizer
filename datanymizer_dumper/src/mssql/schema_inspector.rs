use super::{column::MsSqlColumn, dumper::MsSqlDumper, sql_type::MsSqlType, table::MsSqlTable};
use crate::{Dumper, SchemaInspector, Table};
use anyhow::Result;
use std::collections::HashMap;

#[derive(Clone)]
pub struct MsSqlSchemaInspector;

impl SchemaInspector for MsSqlSchemaInspector {
    type Type = MsSqlType;
    type Dumper = MsSqlDumper;
    type Table = MsSqlTable;
    type Column = MsSqlColumn;

    fn get_tables(&self, c: &mut <Self::Dumper as Dumper>::Connection) -> Result<Vec<Self::Table>> {
        let schema_ids = self.get_schema_ids(c)?;
        let computed_columns = self.get_computed_columns(c)?;

        let tables = Self::Dumper::query(
            c,
            "SELECT TABLE_NAME, TABLE_SCHEMA FROM INFORMATION_SCHEMA.TABLES \
            WHERE TABLE_TYPE='BASE TABLE'",
        )?
        .iter()
        .map(|row| {
            let name = row.get::<&str, _>(0).expect("table name column is missed");
            let schema = row.get::<&str, _>(1).expect("schema name column is missed");
            let schema_id = schema_ids[schema];
            let table_id = self
                .get_table_id(c, name, schema_id)
                .expect("missing table id");
            let table_computed_columns = computed_columns.get(&table_id);
            let has_identity_column = self
                .has_identity_column(c, table_id)
                .expect("missing PK info");

            let mut table = MsSqlTable::new(name, Some(schema), has_identity_column);

            if let Ok(columns) = self.get_columns(c, &table) {
                let columns = columns
                    .into_iter()
                    .filter(|c| match table_computed_columns {
                        Some(cols) => !cols.contains(&c.name),
                        None => true,
                    })
                    .collect();
                table.set_columns(columns);
            };

            match self.get_table_size(c, table.full_escaped_name()) {
                Ok(size) => table.size = size as i64,
                Err(e) => panic!("ERR: {}", e),
            }

            table
        })
        .collect();
        Ok(tables)
    }

    /// Get table size
    fn get_table_size(
        &self,
        c: &mut <Self::Dumper as Dumper>::Connection,
        table_name: String,
    ) -> Result<i64> {
        let count = <MsSqlSchemaInspector as SchemaInspector>::Dumper::query(
            c,
            format!("SELECT COUNT(*) FROM {}", table_name).as_str(),
        )?
        .first()
        .expect("missing table")
        .get::<i64, _>(0)
        .expect("missing count for table");

        Ok(count)
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

impl MsSqlSchemaInspector {
    fn get_computed_columns(
        &self,
        c: &mut <<MsSqlSchemaInspector as SchemaInspector>::Dumper as Dumper>::Connection,
    ) -> Result<HashMap<i32, Vec<String>>> {
        let mut columns = HashMap::new();
        for row in <MsSqlSchemaInspector as SchemaInspector>::Dumper::query(
            c,
            "SELECT name, object_id FROM sys.computed_columns",
        )?
        .iter()
        {
            let column_name = row
                .get::<&str, _>("name")
                .expect("column name is missed")
                .to_string();
            let table_id = row.get::<i32, _>("object_id").expect("table id is missed");
            columns.entry(table_id).or_insert_with(Vec::new);
            columns.get_mut(&table_id).unwrap().push(column_name);
        }

        Ok(columns)
    }

    fn get_schema_ids(
        &self,
        c: &mut <<MsSqlSchemaInspector as SchemaInspector>::Dumper as Dumper>::Connection,
    ) -> Result<HashMap<String, i32>> {
        let mut schema_ids = HashMap::new();
        for row in <MsSqlSchemaInspector as SchemaInspector>::Dumper::query(
            c,
            "SELECT name, schema_id FROM sys.schemas",
        )?
        .iter()
        {
            let name = row
                .get::<&str, _>("name")
                .expect("schema name is missed")
                .to_string();
            let id = row.get::<i32, _>("schema_id").expect("schema id is missed");
            schema_ids.insert(name, id);
        }

        Ok(schema_ids)
    }

    fn get_table_id(
        &self,
        c: &mut <<MsSqlSchemaInspector as SchemaInspector>::Dumper as Dumper>::Connection,
        table: &str,
        schema_id: i32,
    ) -> Result<i32> {
        let id = <MsSqlSchemaInspector as SchemaInspector>::Dumper::query(
            c,
            format!(
                "SELECT object_id FROM sys.tables WHERE schema_id = {} AND name = N'{}'",
                schema_id, table
            )
            .as_str(),
        )?
        .first()
        .expect("missing table")
        .get::<i32, _>(0)
        .expect("missing object_id for table");

        Ok(id)
    }

    fn has_identity_column(
        &self,
        c: &mut <<MsSqlSchemaInspector as SchemaInspector>::Dumper as Dumper>::Connection,
        table_id: i32,
    ) -> Result<bool> {
        let has_identity_column = !<MsSqlSchemaInspector as SchemaInspector>::Dumper::query(
            c,
            format!(
                "SELECT name FROM sys.identity_columns WHERE object_id = {}",
                table_id
            )
            .as_str(),
        )?
        .is_empty();

        Ok(has_identity_column)
    }
}
