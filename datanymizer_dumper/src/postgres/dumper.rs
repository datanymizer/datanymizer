use super::row::PgRow;
use super::schema_inspector::PgSchemaInspector;
use super::table::PgTable;
use crate::{Dumper, SchemaInspector, Table};
use anyhow::Result;
use datanymizer_engine::{Engine, Settings};
use indicatif::{HumanDuration, ProgressBar, ProgressStyle};
use postgres::Client;
use std::{
    fs::{File, OpenOptions},
    io::prelude::*,
    process::Command,
    time::Instant,
};

pub struct PgDumper {
    schema_inspector: PgSchemaInspector,
    engine: Engine,
    dump_writer: File,
    pg_dump_location: String,
}

impl PgDumper {
    pub fn new(engine: Engine, pg_dump_location: String) -> Result<Self> {
        let destination = engine.settings.destination()?;
        let dump_writer = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(destination)?;

        Ok(Self {
            engine,
            dump_writer,
            pg_dump_location,
            schema_inspector: PgSchemaInspector {},
        })
    }
}

impl Dumper for PgDumper {
    type Connection = Client;
    type SchemaInspector = PgSchemaInspector;
    type DumpWriter = File;
    type Table = PgTable;

    fn schema_inspector(&self) -> Self::SchemaInspector {
        self.schema_inspector.clone()
    }

    // Stage before dumping data. It makes dump schema with any options
    fn pre_data(&mut self, _connection: &mut Self::Connection) -> Result<()> {
        println!("Prepare data scheme...");
        let dump_output = Command::new(&self.pg_dump_location)
            .args(&["--section", "pre-data"])
            .arg(self.engine.settings.source.get_database_url())
            .output()?;
        self.dump_writer
            .write_all(&dump_output.stdout)
            .map_err(|e| e.into())
    }

    // This stage makes dump data only
    fn data(&mut self, connection: &mut Self::Connection) -> Result<()> {
        let settings = self.settings();
        self.write_log("Start dumping data".into())?;
        //TODO: Return combined error from results
        let mut tables = self.schema_inspector().ordered_tables(connection);
        tables.sort_by(|a, b| b.1.cmp(&a.1));
        let all_tables_count = tables.len();
        // In transaction
        let mut tr = connection.transaction()?;
        for (ind, (table, _weidth)) in tables.iter().enumerate() {
            println!(
                "[{} / {}] Prepare to dump table: {}",
                ind + 1,
                all_tables_count,
                table.get_full_name()
            );
            if self.filter_table(table.get_full_name(), &settings.filter) {
                let tsize: u64 = table.get_size() as u64;
                let pb = ProgressBar::new(tsize);
                let delta = tsize / 100;
                pb.set_draw_delta(delta);
                pb.set_prefix(&table.get_full_name());
                pb.set_style(ProgressStyle::default_bar().template(
                    "[Dumping: {prefix}] [|{bar:50}|] {pos} of {len} rows [{percent}%] ({eta})",
                ).progress_chars("#>-"));

                let started = Instant::now();

                let qt = table.query_to();
                let reader = tr.copy_out(qt.as_str())?;

                self.write_log(format!("Dump table: {}", &table.get_full_name()))?;

                self.dump_writer.write_all(b"\n")?;
                self.dump_writer.write_all(&table.query_from().as_bytes())?;
                self.dump_writer.write_all(b"\n")?;
                for line in reader.lines() {
                    // Tick for bar
                    pb.inc(1);

                    let l = line?;
                    let row = PgRow::from_string_row(l.to_string(), table.clone());
                    let transformed = row.transform(&self.engine)?;
                    // Writer::from_writer(&self.dump_writer).write_record(&transformed_row)?;
                    self.dump_writer.write_all(transformed.as_bytes())?;
                    self.dump_writer.write_all(b"\n")?;
                }
                self.dump_writer.write_all(b"\\.\n")?;
                pb.finish();
                let finished = started.elapsed();
                println!(
                    "[Dumping: {}] Finished in {}",
                    table.get_full_name(),
                    HumanDuration(finished)
                );
            } else {
                println!("[Dumping: {}] --- SKIP ---", table.get_full_name());
            }
        }

        self.write_log("End dumping data".into())?;
        Ok(())
    }

    // This stage mekes dump foreign keys, indeces and other...
    fn post_data(&mut self, _connection: &mut Self::Connection) -> Result<()> {
        println!("Finishing with indexes...");
        let dump_output = Command::new(&self.pg_dump_location)
            .args(&["--section", "post-data"])
            .arg(self.engine.settings.source.get_database_url())
            .output()?;
        self.dump_writer
            .write_all(&dump_output.stdout)
            .map_err(|e| e.into())
    }

    fn settings(&mut self) -> Settings {
        self.engine.settings.clone()
    }

    fn write_log(&mut self, message: String) -> Result<()> {
        self.dump_writer
            .write_all(format!("\n---\n--- {}\n---\n", message).as_bytes())
            .map_err(|e| e.into())
    }
}
