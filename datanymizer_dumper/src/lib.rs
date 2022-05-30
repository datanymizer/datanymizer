use anyhow::Result;
use core::iter::Iterator;
use datanymizer_engine::{Filter, Settings};
use indicatif::HumanDuration;
use solvent::DepGraph;
use std::{collections::HashMap, hash::Hash, time::Instant};

pub mod indicator;
pub mod postgres;

// Dumper makes dump with same stages
pub trait Dumper: 'static + Sized + Send {
    type Connection;
    type SchemaInspector: SchemaInspector<Connection = Self::Connection>;

    /// Process steps
    fn dump(&mut self, connection: &mut Self::Connection) -> Result<()> {
        let started = Instant::now();
        self.debug("Fetching tables metadata...".into());
        self.prepare(connection)?;
        self.pre_data(connection)?;
        self.data(connection)?;
        self.post_data(connection)?;

        let finished = started.elapsed();
        self.debug(format!("Full Dump finished in {}", HumanDuration(finished)));
        Ok(())
    }

    /// Preparation (load table list, set table ordering, initialize table filter)
    fn prepare(&mut self, connection: &mut Self::Connection) -> Result<()> {
        let mut tables = self.schema_inspector().ordered_tables(connection)?;
        sort_tables(&mut tables, &self.settings().table_order);
        let tables: Vec<_> = tables.into_iter().map(|(t, _)| t).collect();
        self.filter_mut()
            .load_tables(tables.iter().map(|t| t.get_full_name()).collect());
        self.set_tables(tables);

        Ok(())
    }

    /// Stage before dumping data. It makes dump schema with any options
    fn pre_data(&mut self, _connection: &mut Self::Connection) -> Result<()>;

    fn data(&mut self, connection: &mut Self::Connection) -> Result<()>;

    /// This stage makes dump foreign keys, indices and other...
    fn post_data(&mut self, _connection: &mut Self::Connection) -> Result<()>;

    fn filter_table(&self, table: String) -> bool {
        self.settings().filter.filter_table(&table)
    }

    fn schema_inspector(&self) -> Self::SchemaInspector;

    fn set_tables(&mut self, tables: Vec<<Self::SchemaInspector as SchemaInspector>::Table>);

    fn settings(&self) -> &Settings;

    fn filter_mut(&mut self) -> &mut Filter;

    fn write_log(&mut self, message: String) -> Result<()>;

    fn debug(&self, message: String);
}

fn sort_tables<T, Tbl: Table<T>>(tables: &mut [(Tbl, i32)], order: &[String]) {
    tables.sort_by_cached_key(|(tbl, weight)| {
        let position = order.iter().position(|i| tbl.get_names().contains(i));
        (position, -weight)
    });
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

        let tables = self.get_tables(connection)?;
        let mut weight_map: HashMap<String, i32> = HashMap::with_capacity(tables.len());

        for table in tables.iter() {
            depgraph.register_dependencies(table.get_full_name(), table.get_dep_table_names());
        }

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

#[cfg(test)]
mod tests {
    use super::*;
    use datanymizer_engine::TableList;
    use std::hash::Hasher;

    struct MockColumn;

    impl ColumnData<()> for MockColumn {
        fn position(&self) -> usize {
            0
        }

        fn name(&self) -> &str {
            ""
        }

        fn inner_kind(&self) -> Option<()> {
            None
        }
    }

    #[derive(Debug, Clone, Eq, PartialEq)]
    struct MockTable {
        schema: &'static str,
        name: &'static str,
        col_map: HashMap<String, usize>,
    }

    impl MockTable {
        fn new(schema: &'static str, name: &'static str) -> Self {
            let col_map = HashMap::new();
            Self {
                schema,
                name,
                col_map,
            }
        }
    }

    impl Hash for MockTable {
        fn hash<H: Hasher>(&self, state: &mut H) {
            self.schema.hash(state);
            self.name.hash(state);
        }
    }

    impl Table<()> for MockTable {
        type Column = MockColumn;
        type Row = ();

        fn get_name(&self) -> String {
            self.name.to_string()
        }

        fn get_full_name(&self) -> String {
            format!("{}.{}", self.schema, self.name)
        }

        fn get_names(&self) -> Vec<String> {
            vec![self.get_full_name(), self.get_name()]
        }

        fn get_columns(&self) -> Vec<Self::Column> {
            Vec::new()
        }

        fn get_columns_names(&self) -> Vec<String> {
            Vec::new()
        }

        fn get_size(&self) -> i64 {
            0
        }

        fn get_column_indexes(&self) -> &HashMap<String, usize> {
            &self.col_map
        }

        fn get_dep_table_names(&self) -> Vec<String> {
            vec![]
        }
    }

    struct MockConnection;

    #[derive(Clone)]
    struct MockSchemaInspector;

    impl SchemaInspector for MockSchemaInspector {
        type Type = ();
        type Connection = MockConnection;
        type Table = MockTable;
        type Column = MockColumn;
        type ForeignKey = ();

        fn get_tables(&self, _connection: &mut Self::Connection) -> Result<Vec<Self::Table>> {
            Ok(vec![
                MockTable::new("public", "table1"),
                MockTable::new("other", "table1"),
                MockTable::new("public", "table2"),
                MockTable::new("public", "table3"),
            ])
        }

        fn get_table_size(
            &self,
            _connection: &mut Self::Connection,
            _table: &Self::Table,
        ) -> Result<i64> {
            Ok(4)
        }

        fn get_foreign_keys(
            &self,
            _connection: &mut Self::Connection,
            _table: &Self::Table,
        ) -> Result<Vec<Self::ForeignKey>> {
            Ok(vec![])
        }

        fn get_columns(
            &self,
            _connection: &mut Self::Connection,
            _table: &Self::Table,
        ) -> Result<Vec<Self::Column>> {
            Ok(vec![])
        }
    }

    struct MockDumper {
        pub tables: Vec<MockTable>,
        pub settings: Settings,
    }

    impl MockDumper {
        fn new(cfg: &str) -> Self {
            let tables = vec![];
            let settings = Settings::from_yaml(cfg).unwrap();
            Self { tables, settings }
        }
    }

    impl Dumper for MockDumper {
        type Connection = MockConnection;
        type SchemaInspector = MockSchemaInspector;

        fn pre_data(&mut self, _connection: &mut Self::Connection) -> Result<()> {
            Ok(())
        }

        fn data(&mut self, _connection: &mut Self::Connection) -> Result<()> {
            Ok(())
        }

        fn post_data(&mut self, _connection: &mut Self::Connection) -> Result<()> {
            Ok(())
        }

        fn schema_inspector(&self) -> Self::SchemaInspector {
            MockSchemaInspector
        }

        fn set_tables(&mut self, tables: Vec<<Self::SchemaInspector as SchemaInspector>::Table>) {
            self.tables = tables;
        }

        fn settings(&self) -> &Settings {
            &self.settings
        }

        fn filter_mut(&mut self) -> &mut Filter {
            &mut self.settings.filter
        }

        fn write_log(&mut self, _message: String) -> Result<()> {
            Ok(())
        }

        fn debug(&self, _message: String) {}
    }

    #[test]
    fn test_sort_tables() {
        let order = vec!["table2".to_string(), "public.table1".to_string()];

        let mut tables = vec![
            (MockTable::new("public", "table1"), 0),
            (MockTable::new("public", "table2"), 1),
            (MockTable::new("public", "table3"), 2),
            (MockTable::new("public", "table4"), 3),
            (MockTable::new("other", "table1"), 4),
            (MockTable::new("other", "table2"), 5),
        ];

        sort_tables(&mut tables, &order);

        let ordered_names: Vec<_> = tables
            .iter()
            .map(|(t, w)| (t.get_full_name(), *w))
            .collect();
        assert_eq!(
            ordered_names,
            vec![
                ("other.table1".to_string(), 4),
                ("public.table4".to_string(), 3),
                ("public.table3".to_string(), 2),
                ("other.table2".to_string(), 5),
                ("public.table2".to_string(), 1),
                ("public.table1".to_string(), 0),
            ]
        )
    }

    #[test]
    fn prepare() {
        let cfg = r#"
            table_order:
              - table3
              - other.table1
              - public.table1
            filter:
              schema:
                except:
                  - other.*
              data:
                only:
                  - table1
                  - table2
            "#;
        let mut dumper = MockDumper::new(cfg);

        assert!(dumper.prepare(&mut MockConnection).is_ok());
        // it loads sorted tables
        assert_eq!(
            dumper
                .tables
                .iter()
                .map(|t| t.get_full_name())
                .collect::<Vec<_>>(),
            vec![
                "public.table2",
                "public.table3",
                "other.table1",
                "public.table1"
            ]
        );
        // it inits filter
        assert_eq!(
            dumper.settings.filter.schema_match_list(),
            &TableList::Except(vec![String::from("other.table1")])
        );
    }
}
