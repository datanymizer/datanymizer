use anyhow::Result;
use core::iter::Iterator;
use datanymizer_engine::{Filter, Settings};
use indicatif::HumanDuration;
use solvent::DepGraph;
use std::{collections::HashMap, hash::Hash, time::Instant};

pub mod postgres;

// Dumper makes dump with same stages
pub trait Dumper: 'static + Sized + Send {
    type Table;
    type Connection;
    type SchemaInspector: SchemaInspector<Dumper = Self>;

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

    fn filter_table(&mut self, table: String, filter: &Option<Filter>) -> bool {
        if let Some(f) = filter {
            f.filter_schema(&table) && f.filter_data(&table)
        } else {
            true
        }
    }

    fn schema_inspector(&self) -> Self::SchemaInspector;

    /// This stage makes dump foreign keys, indices and other...
    fn post_data(&mut self, _connection: &mut Self::Connection) -> Result<()>;

    fn settings(&mut self) -> Settings;

    fn write_log(&mut self, message: String) -> Result<()>;

    fn debug(&self, message: String);
}

pub trait SchemaInspector: 'static + Sized + Send + Clone {
    type Type;
    type Dumper: Dumper;
    type Table: Table<Self::Type>;
    type Column: ColumnData<Self::Type>;

    /// Get all tables in the database
    fn get_tables(
        &self,
        connection: &mut <Self::Dumper as Dumper>::Connection,
    ) -> Result<Vec<Self::Table>>;

    /// Get table size
    fn get_table_size(
        &self,
        connection: &mut <Self::Dumper as Dumper>::Connection,
        table_name: String,
    ) -> Result<i64>;

    /// Get all dependencies (by FK) for `table` in database
    fn get_dependencies(
        &self,
        connection: &mut <Self::Dumper as Dumper>::Connection,
        table: &Self::Table,
    ) -> Result<Vec<Self::Table>>;

    fn ordered_tables(
        &self,
        connection: &mut <Self::Dumper as Dumper>::Connection,
    ) -> Vec<(Self::Table, i32)> {
        let mut res: HashMap<Self::Table, i32> = HashMap::new();
        let mut depgraph: DepGraph<Self::Table> = DepGraph::new();
        if let Ok(tables) = self.get_tables(connection) {
            for table in tables.iter() {
                let deps: Vec<Self::Table> = self
                    .get_dependencies(connection, &table)
                    .unwrap_or_default()
                    .into_iter()
                    .collect();
                depgraph.register_dependencies(table.clone(), deps);
            }

            for table in tables.iter() {
                let _ = res.entry(table.clone()).or_insert(0);
                if let Ok(nodes) = depgraph.dependencies_of(&table) {
                    for node in nodes {
                        if let Ok(node) = node {
                            let counter = res.entry(node.clone()).or_insert(0);
                            *counter += 1;
                        }
                    }
                }
            }
        }
        res.iter().map(|(k, b)| (k.clone(), *b)).collect()
    }

    /// Get columns for table
    fn get_columns(
        &self,
        connection: &mut <Self::Dumper as Dumper>::Connection,
        table: &Self::Table,
    ) -> Result<Vec<Self::Column>>;
}

/// Table trait for all databases
pub trait Table<T>: Sized + Send + Clone + Eq + Hash {
    type Dumper: Dumper;
    type Column: ColumnData<T>;
    type Row;

    /// Returns table name
    fn get_name(&self) -> String;
    /// Returns table name with schema or other prefix, based on database type
    fn get_full_name(&self) -> String;
    /// Get table columns
    fn get_columns(&self) -> Vec<Self::Column>;
    /// Get columns names (needed in the future for SQL)
    fn get_columns_names(&self) -> Vec<String>;
    /// Get table size
    fn get_size(&self) -> i64;
    /// Get column name - index map
    fn get_column_indexes(&self) -> &HashMap<String, usize>;
}

pub trait ColumnData<T> {
    fn position(&self) -> usize;
    fn name(&self) -> &str;
    fn inner_kind(&self) -> Option<T>;
}
