use super::{
    connector, query_wrapper::QueryWrapper, row::PgRow, schema_inspector::PgSchemaInspector,
    table::PgTable,
};
use crate::{indicator::Indicator, Dumper, SchemaInspector, Table};
use anyhow::Result;
use datanymizer_engine::{Engine, Filter, Settings, TableList};
use postgres::IsolationLevel;
use std::{
    io::{self, prelude::*},
    process::{self, Command},
    time::Instant,
};

pub struct PgDumper<W: Write + Send, I: Indicator + Send> {
    schema_inspector: PgSchemaInspector,
    engine: Engine,
    dump_writer: W,
    indicator: I,
    dump_isolation_level: Option<IsolationLevel>,
    pg_dump_location: String,
    pg_dump_args: Vec<String>,
    tables: Vec<PgTable>,
}

impl<W: 'static + Write + Send, I: 'static + Indicator + Send> PgDumper<W, I> {
    pub fn new(
        engine: Engine,
        dump_isolation_level: Option<IsolationLevel>,
        pg_dump_location: String,
        dump_writer: W,
        indicator: I,
        pg_dump_args: Vec<String>,
    ) -> Result<Self> {
        Ok(Self {
            engine,
            dump_writer,
            indicator,
            dump_isolation_level,
            pg_dump_location,
            schema_inspector: PgSchemaInspector {},
            pg_dump_args,
            tables: Vec::new(),
        })
    }

    fn run_pg_dump(&mut self, section: &str, db_url: &str) -> Result<()> {
        let program = &self.pg_dump_location;
        let args = vec!["--section", section];
        let table_args = table_args(&self.engine.settings.filter)?;

        let dump_output = Command::new(program)
            .args(&self.pg_dump_args)
            .args(&args)
            .args(&table_args)
            .arg(&db_url)
            .output()?;
        if !dump_output.status.success() {
            eprintln!(
                "pg_dump error. Command:\n{} {} {}\nOutput:",
                program,
                args.into_iter()
                    .chain(table_args.iter().map(|s| s.as_str()))
                    .collect::<Vec<_>>()
                    .join(" "),
                db_url
            );

            io::stderr().write_all(&dump_output.stderr)?;
            process::exit(1);
        }

        self.dump_writer
            .write_all(&dump_output.stdout)
            .map_err(|e| e.into())
    }

    fn dump_table(&mut self, table: &PgTable, qw: &mut QueryWrapper) -> Result<()> {
        let started = Instant::now();

        self.write_log(format!("Dump table: {}", &table.get_full_name()))?;

        self.dump_writer.write_all(b"\n")?;
        self.dump_writer.write_all(table.query_from().as_bytes())?;
        self.dump_writer.write_all(b"\n")?;

        let cfg = self.engine.settings.find_table(&table.get_names());

        self.indicator
            .start_pb(table.count_of_query_to(cfg), &table.get_full_name());

        let mut count: u64 = 0;
        if let Some(cfg) = cfg {
            if let Some(transformed_query) = table.transformed_query_to(Some(cfg), count) {
                let reader = qw.copy_out(transformed_query.as_str())?;
                for line in reader.lines() {
                    self.indicator.inc_pb(1);

                    let row = PgRow::from_string_row(line?, table.clone());
                    let transformed = row.transform(&self.engine, cfg.name.as_str())?;
                    self.dump_writer.write_all(transformed.as_bytes())?;
                    self.dump_writer.write_all(b"\n")?;

                    count += 1;
                }
            }
        }

        if let Some(untransformed_query) = table.untransformed_query_to(cfg, count) {
            let reader = qw.copy_out(untransformed_query.as_str())?;
            for line in reader.lines() {
                self.indicator.inc_pb(1);

                self.dump_writer.write_all(line?.as_bytes())?;
                self.dump_writer.write_all(b"\n")?;

                count += 1;
            }
        }

        self.dump_writer.write_all(b"\\.\n")?;
        for seq in &table.sequences {
            let last_value: i64 = qw.query_one(seq.last_value_query().as_str(), &[])?.get(0);
            self.dump_writer.write_all(b"\n")?;
            self.dump_writer
                .write_all(seq.setval_query(last_value).as_bytes())?;
            self.dump_writer.write_all(b"\n")?;
        }

        let finished = started.elapsed();
        self.indicator
            .finish_pb(table.get_full_name().as_str(), finished);

        Ok(())
    }
}

impl<W: 'static + Write + Send, I: 'static + Indicator + Send> Dumper for PgDumper<W, I> {
    type Table = PgTable;
    type Connection = connector::Connection;
    type SchemaInspector = PgSchemaInspector;

    // Stage before dumping data. It makes dump schema with any options
    fn pre_data(&mut self, connection: &mut Self::Connection) -> Result<()> {
        self.debug("Fetch tables metadata...".into());
        let mut tables = self.schema_inspector().ordered_tables(connection);

        sort_tables(
            &mut tables,
            self.engine.settings.table_order.as_ref().unwrap_or(&vec![]),
        );
        self.tables = tables.into_iter().map(|(t, _)| t).collect();

        if let Some(filter) = &mut self.engine.settings.filter {
            filter.load_tables(self.tables.iter().map(|t| t.get_full_name()).collect());
        }

        self.debug("Prepare data scheme...".into());
        self.run_pg_dump("pre-data", connection.url.as_str())
    }

    // This stage makes dump data only
    fn data(&mut self, connection: &mut Self::Connection) -> Result<()> {
        self.write_log("Start dumping data".into())?;

        let all_tables_count = self.tables.len();

        let mut query_wrapper =
            QueryWrapper::with_isolation_level(&mut connection.client, self.dump_isolation_level)?;
        for (ind, table) in self.tables.clone().iter().enumerate() {
            self.debug(format!(
                "[{} / {}] Prepare to dump table: {}",
                ind + 1,
                all_tables_count,
                table.get_full_name(),
            ));

            if self.filter_table(table.get_full_name()) {
                self.dump_table(table, &mut query_wrapper)?;
            } else {
                self.debug(format!("[Dumping: {}] --- SKIP ---", table.get_full_name()));
            }
        }

        self.write_log("End dumping data".into())?;
        Ok(())
    }

    // This stage makes dump foreign keys, indices and other...
    fn post_data(&mut self, connection: &mut Self::Connection) -> Result<()> {
        self.debug("Finishing with indexes...".into());
        self.run_pg_dump("post-data", connection.url.as_str())
    }

    fn schema_inspector(&self) -> Self::SchemaInspector {
        self.schema_inspector.clone()
    }

    fn settings(&self) -> &Settings {
        &self.engine.settings
    }

    fn write_log(&mut self, message: String) -> Result<()> {
        self.dump_writer
            .write_all(format!("\n---\n--- {}\n---\n", message).as_bytes())
            .map_err(|e| e.into())
    }

    fn debug(&self, message: String) {
        self.indicator.debug_msg(message.as_str());
    }
}

fn table_args(filter: &Option<Filter>) -> Result<Vec<String>> {
    let mut args = vec![];
    if let Some(f) = filter {
        let list = f.schema_match_list();
        let flag = match list {
            TableList::Only(_) => "-t",
            TableList::Except(_) => "-T",
        };
        for table in list.tables() {
            args.push(String::from(flag));
            args.push(PgTable::quote_table_name(table.as_str())?);
        }
    }

    Ok(args)
}

fn sort_tables(tables: &mut Vec<(PgTable, i32)>, order: &[String]) {
    tables.sort_by_cached_key(|(tbl, weight)| {
        let position = order.iter().position(|i| tbl.get_names().contains(i));
        (position, -weight)
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_table_args() {
        let tables = vec![String::from("table1"), String::from("table2")];

        let empty: Vec<String> = vec![];
        assert_eq!(table_args(&None).unwrap(), empty);

        let mut filter = Filter::new(
            TableList::Except(vec![String::from("table1")]),
            TableList::default(),
        );
        filter.load_tables(tables.clone());
        assert_eq!(
            table_args(&Some(filter)).unwrap(),
            vec![String::from("-T"), String::from("\"table1\"")]
        );

        let mut filter = Filter::new(
            TableList::default(),
            TableList::Except(vec![String::from("table1")]),
        );
        filter.load_tables(tables.clone());
        assert_eq!(table_args(&Some(filter)).unwrap(), empty);

        let mut filter = Filter::new(
            TableList::Only(vec![String::from("table1"), String::from("table2")]),
            TableList::default(),
        );
        filter.load_tables(tables.clone());
        assert_eq!(
            table_args(&Some(filter)).unwrap(),
            vec![
                String::from("-t"),
                String::from("\"table1\""),
                String::from("-t"),
                String::from("\"table2\"")
            ]
        );

        let mut filter = Filter::new(
            TableList::Only(vec![String::from("table*")]),
            TableList::default(),
        );
        filter.load_tables(tables);
        assert_eq!(
            table_args(&Some(filter)).unwrap(),
            vec![
                String::from("-t"),
                String::from("\"table1\""),
                String::from("-t"),
                String::from("\"table2\"")
            ]
        );
    }

    #[test]
    fn test_sort_tables() {
        let order = vec!["table2".to_string(), "public.table1".to_string()];

        let mut tables = vec![
            (PgTable::new("table1".to_string(), "public".to_string()), 0),
            (PgTable::new("table2".to_string(), "public".to_string()), 1),
            (PgTable::new("table3".to_string(), "public".to_string()), 2),
            (PgTable::new("table4".to_string(), "public".to_string()), 3),
            (PgTable::new("table1".to_string(), "other".to_string()), 4),
            (PgTable::new("table2".to_string(), "other".to_string()), 5),
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
}
