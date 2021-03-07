use datanymizer_engine::Engine;
use indicatif::ProgressBar;

pub struct MsSqlDumper {
    schema_inspector: MsSqlSchemaInspector,
    engine: Engine,
    mssql_scripter_location: String,
    progress_bar: ProgressBar,
}

impl Dumper for MsSqlDumper {
    type Connection = Client;
    type SchemaInspector = MsSqlSchemaInspector;
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
