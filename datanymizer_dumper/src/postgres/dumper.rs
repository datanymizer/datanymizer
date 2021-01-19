use super::row::PgRow;
use super::schema_inspector::PgSchemaInspector;
use super::table::PgTable;
use super::writer::DumpWriter;
use crate::{Dumper, SchemaInspector, Table};
use anyhow::Result;
use datanymizer_engine::{Engine, Filter, Settings, TableList};
use indicatif::{HumanDuration, ProgressBar, ProgressStyle};
use postgres::Client;
use std::{io::prelude::*, process::Command, time::Instant};

pub struct PgDumper {
    schema_inspector: PgSchemaInspector,
    engine: Engine,
    dump_writer: DumpWriter,
    pg_dump_location: String,
    progress_bar: ProgressBar,
}

impl PgDumper {
    pub fn new(engine: Engine, pg_dump_location: String, target: Option<String>) -> Result<Self> {
        let dump_writer = DumpWriter::new(target)?;
        let pb: ProgressBar = if dump_writer.can_log_to_stdout() {
            ProgressBar::new(0)
        } else {
            ProgressBar::hidden()
        };
        Ok(Self {
            engine,
            dump_writer,
            pg_dump_location,
            schema_inspector: PgSchemaInspector {},
            progress_bar: pb,
        })
    }

    fn init_progress_bar(&self, tsize: u64, prefix: &str) {
        let delta = tsize / 100;
        self.progress_bar.set_length(tsize);
        self.progress_bar.set_draw_delta(delta);
        self.progress_bar.set_prefix(prefix);
        self.progress_bar.set_style(
            ProgressStyle::default_bar()
                .template(
                    "[Dumping: {prefix}] [|{bar:50}|] {pos} of {len} rows [{percent}%] ({eta})",
                )
                .progress_chars("#>-"),
        );
    }

    fn table_args(filter: &Option<Filter>) -> Vec<String> {
        if let Some(f) = filter {
            if let Some(list) = &f.schema {
                let flag = match list {
                    TableList::Only(_) => "-t",
                    TableList::Except(_) => "-T",
                };

                return list
                    .tables()
                    .iter()
                    .flat_map(|table| vec![String::from(flag), table.clone()])
                    .collect();
            }
        }

        Vec::new()
    }
}

impl Dumper for PgDumper {
    type Connection = Client;
    type SchemaInspector = PgSchemaInspector;
    type Table = PgTable;

    fn schema_inspector(&self) -> Self::SchemaInspector {
        self.schema_inspector.clone()
    }

    // Stage before dumping data. It makes dump schema with any options
    fn pre_data(&mut self, _connection: &mut Self::Connection) -> Result<()> {
        self.debug("Prepare data scheme...".into());
        let dump_output = Command::new(&self.pg_dump_location)
            .args(&["--section", "pre-data"])
            .args(Self::table_args(&self.engine.settings.filter))
            .arg(self.engine.settings.source.get_database_url())
            .output()?;

        self.dump_writer
            .write_all(&dump_output.stdout)
            .map_err(|e| e)
    }

    // This stage makes dump data only
    fn data(&mut self, connection: &mut Self::Connection) -> Result<()> {
        let settings = self.settings();
        self.write_log("Start dumping data".into())?;
        self.debug("Fetch tables metadata...".into());
        let mut tables = self.schema_inspector().ordered_tables(connection);
        tables.sort_by(|a, b| b.1.cmp(&a.1));
        let all_tables_count = tables.len();
        // In transaction
        let mut tr = connection.transaction()?;
        for (ind, (table, _weidth)) in tables.iter().enumerate() {
            self.debug(format!(
                "[{} / {}] Prepare to dump table: {}",
                ind + 1,
                all_tables_count,
                table.get_full_name(),
            ));
            if self.filter_table(table.get_full_name(), &settings.filter) {
                let started = Instant::now();

                self.init_progress_bar(table.get_size() as u64, &table.get_full_name());

                let qt = table.query_to();
                let reader = tr.copy_out(qt.as_str())?;

                self.write_log(format!("Dump table: {}", &table.get_full_name()))?;

                self.dump_writer.write_all(b"\n")?;
                self.dump_writer.write_all(&table.query_from().as_bytes())?;
                self.dump_writer.write_all(b"\n")?;
                for line in reader.lines() {
                    // Tick for bar
                    self.progress_bar.inc(1);

                    let l = line?;
                    let row = PgRow::from_string_row(l.to_string(), table.clone());
                    let transformed = row.transform(&self.engine)?;
                    // Writer::from_writer(&self.dump_writer).write_record(&transformed_row)?;
                    self.dump_writer.write_all(transformed.as_bytes())?;
                    self.dump_writer.write_all(b"\n")?;
                }
                self.dump_writer.write_all(b"\\.\n")?;
                self.progress_bar.finish();
                self.progress_bar.reset();

                let finished = started.elapsed();
                self.debug(format!(
                    "[Dumping: {}] Finished in {}",
                    table.get_full_name(),
                    HumanDuration(finished),
                ));
            } else {
                self.debug(format!("[Dumping: {}] --- SKIP ---", table.get_full_name()));
            }
        }

        self.write_log("End dumping data".into())?;
        Ok(())
    }

    // This stage makes dump foreign keys, indices and other...
    fn post_data(&mut self, _connection: &mut Self::Connection) -> Result<()> {
        self.debug("Finishing with indexes...".into());
        let dump_output = Command::new(&self.pg_dump_location)
            .args(&["--section", "post-data"])
            .args(Self::table_args(&self.engine.settings.filter))
            .arg(self.engine.settings.source.get_database_url())
            .output()?;

        self.dump_writer
            .write_all(&dump_output.stdout)
            .map_err(|e| e)
    }

    fn settings(&mut self) -> Settings {
        self.engine.settings.clone()
    }

    fn write_log(&mut self, message: String) -> Result<()> {
        self.dump_writer
            .write_all(format!("\n---\n--- {}\n---\n", message).as_bytes())
            .map_err(|e| e)
    }

    fn debug(&self, message: String) {
        if self.dump_writer.can_log_to_stdout() {
            println!("{}", message)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn table_args() {
        let empty: Vec<String> = vec![];
        assert_eq!(PgDumper::table_args(&None), empty);

        let filter = Filter {
            schema: Some(TableList::Except(vec![String::from("table1")])),
            data: None,
        };
        assert_eq!(
            PgDumper::table_args(&Some(filter)),
            vec![String::from("-T"), String::from("table1")]
        );

        let filter = Filter {
            schema: None,
            data: Some(TableList::Except(vec![String::from("table1")])),
        };
        assert_eq!(PgDumper::table_args(&Some(filter)), empty);

        let filter = Filter {
            schema: Some(TableList::Only(vec![
                String::from("table1"),
                String::from("table2"),
            ])),
            data: None,
        };
        assert_eq!(
            PgDumper::table_args(&Some(filter)),
            vec![
                String::from("-t"),
                String::from("table1"),
                String::from("-t"),
                String::from("table2")
            ]
        );
    }
}
