use anyhow::Result;
use core::iter::Iterator;
use datanymizer_engine::Settings;
use indicatif::HumanDuration;
use solvent::DepGraph;
use std::{collections::HashMap, hash::Hash, time::Instant};

pub mod indicator;
pub mod postgres;

// Dumper makes dump with same stages
pub trait Dumper: 'static + Sized + Send {
    type Table;
    type Connection;
    type SchemaInspector: SchemaInspector<Connection = Self::Connection>;

    /// Process steps
    fn dump(&mut self, connection: &mut Self::Connection) -> Result<()> {
        let started = Instant::now();
        self.pre_data(connection)?;
        self.data(connection)?;
        self.post_data(connection)?;

        let finished = started.elapsed();
        self.debug(format!("Full Dump finished in {}", HumanDuration(finished)));
        Ok(())
    }

    /// Stage before dumping data. It makes dump schema with any options
    fn pre_data(&mut self, _connection: &mut Self::Connection) -> Result<()>;

    fn data(&mut self, connection: &mut Self::Connection) -> Result<()>;

    /// This stage makes dump foreign keys, indices and other...
    fn post_data(&mut self, _connection: &mut Self::Connection) -> Result<()>;

    fn filter_table(&self, table: String) -> bool {
        self.settings()
            .filter
            .as_ref()
            .map(|f| f.filter_table(&table))
            .unwrap_or(true)
    }

    fn schema_inspector(&self) -> Self::SchemaInspector;

    fn settings(&self) -> &Settings;

    fn write_log(&mut self, message: String) -> Result<()>;

    fn debug(&self, message: String);
}

pub trait SchemaInspector: 'static + Sized + Send + Clone {
    type Type;
    type Connection;
    type Table: Table<Self::Type>;
    type Column: ColumnData<Self::Type>;
    type ForeignKey;

    /// Get all tables in the database
    fn get_tables(&self, connection: &mut Self::Connection) -> Result<Vec<Self::Table>>;

    /// Get table size
    fn get_table_size(&self, connection: &mut Self::Connection, table: &Self::Table)
        -> Result<i64>;

    /// Get foreign keys for table
    fn get_foreign_keys(
        &self,
        connection: &mut Self::Connection,
        table: &Self::Table,
    ) -> Result<Vec<Self::ForeignKey>>;

    fn ordered_tables(&self, connection: &mut Self::Connection) -> Result<Vec<(Self::Table, i32)>> {
        let mut depgraph: DepGraph<String> = DepGraph::new();

        let started = Instant::now();
        println!("Inspecting tables ...");

        let tables = self.get_tables(connection)?;
        let mut weight_map: HashMap<String, i32> = HashMap::with_capacity(tables.len());

        println!(
            "Inspecting completed in {}",
            HumanDuration(started.elapsed())
        );

        println!("Inspecting table dependencies...");
        let started = Instant::now();
        for table in tables.iter() {
            depgraph.register_dependencies(table.get_full_name(), table.get_dep_table_names());
        }

        println!(
            "Inspecting completed in {}",
            HumanDuration(started.elapsed())
        );

        println!("Processing table dependencies...");
        let started = Instant::now();

        for table in tables.iter() {
            let name = table.get_full_name();
            weight_map.entry(name.clone()).or_insert(0);
            if let Ok(dep_names) = depgraph.dependencies_of(&name) {
                for dep_name in dep_names.flatten() {
                    let weight = weight_map.entry(dep_name.clone()).or_insert(0);
                    *weight += 1;
                }
            }
        }

        println!(
            "Processing completed in {}",
            HumanDuration(started.elapsed())
        );

        Ok(tables
            .into_iter()
            .map(|t| {
                let name = t.get_full_name();
                (
                    t,
                    weight_map.get(name.as_str()).copied().unwrap_or_default(),
                )
            })
            .collect())
    }

    /// Get columns for table
    fn get_columns(
        &self,
        connection: &mut Self::Connection,
        table: &Self::Table,
    ) -> Result<Vec<Self::Column>>;
}

/// Table trait for all databases
pub trait Table<T>: Sized + Send + Clone + Eq + Hash {
    type Column: ColumnData<T>;
    type Row;

    /// Returns table name
    fn get_name(&self) -> String;
    /// Returns table name with schema or other prefix, based on database type
    fn get_full_name(&self) -> String;
    /// Returns possible table names (e.g. full and short)
    fn get_names(&self) -> Vec<String>;
    /// Get table columns
    fn get_columns(&self) -> Vec<Self::Column>;
    /// Get columns names (needed in the future for SQL)
    fn get_columns_names(&self) -> Vec<String>;
    /// Get table size
    fn get_size(&self) -> i64;
    /// Get column name - index map
    fn get_column_indexes(&self) -> &HashMap<String, usize>;
    /// Get depended table names
    fn get_dep_table_names(&self) -> Vec<String>;
}

pub trait ColumnData<T> {
    fn position(&self) -> usize;
    fn name(&self) -> &str;
    fn inner_kind(&self) -> Option<T>;
}
