use super::{
    schema_inspector::MsSqlSchemaInspector, scripter::SchemaDump, table::MsSqlTable, value::Value,
};
use crate::{progress_bar::DumpProgressBar, writer::DumpWriter, Dumper, SchemaInspector, Table};
use anyhow::Result;
use async_std::{net::TcpStream, stream::StreamExt, task};
use datanymizer_engine::{Engine, Settings};
use indicatif::ProgressBar;
use std::process::Command;
use tiberius::{Client, Row};

// https://docs.microsoft.com/en-us/sql/t-sql/queries/table-value-constructor-transact-sql?view=sql-server-ver15#limitations-and-restrictions
const BATCH_SIZE: usize = 1000;

pub struct MsSqlDumper {
    schema_inspector: MsSqlSchemaInspector,
    engine: Engine,
    dump_writer: DumpWriter,
    mssql_scripter_location: String,
    progress_bar: ProgressBar,
    schema_dump: Option<SchemaDump>,
}

impl MsSqlDumper {
    pub fn new(
        engine: Engine,
        mssql_scripter_location: String,
        target: Option<String>,
    ) -> Result<Self> {
        let dump_writer = DumpWriter::new(target)?;
        let progress_bar = Self::new_progress_bar(dump_writer.can_log_to_stdout());

        Ok(Self {
            engine,
            dump_writer,
            mssql_scripter_location,
            schema_inspector: MsSqlSchemaInspector,
            progress_bar,
            schema_dump: None,
        })
    }

    pub fn query(c: &mut <Self as Dumper>::Connection, q: &str) -> Result<Vec<Row>> {
        task::block_on(async { c.simple_query(q).await?.into_first_result().await })
            .map_err(anyhow::Error::from)
    }

    fn load_schema_dump(&mut self) -> Result<()> {
        let dump_output = Command::new(&self.mssql_scripter_location)
            // .args(Self::table_args(&self.engine.settings.filter))
            .args(&[
                "--connection-string",
                self.engine.settings.source.get_database_url().as_str(),
            ])
            .output()?;

        self.schema_dump = Some(SchemaDump::new(dump_output.stdout));

        Ok(())
    }

    fn dump_table(
        &mut self,
        c: &mut <Self as Dumper>::Connection,
        table: &MsSqlTable,
    ) -> Result<()> {
        self.debug(format!("Dumping {}...", table.get_full_name()));
        self.write_log(format!("Data for {}", table.get_full_name()))?;
        self.dump_writer
            .write_all(format!("PRINT N'Restoring {}...'\r\n", table.get_full_name()).as_bytes())?;

        let settings = self.settings();
        let cfg = settings.get_table(table.get_full_name().as_str());
        let query_from = table.query_from();

        self.init_progress_bar(table.get_size() as u64, &table.get_full_name());

        task::block_on(async {
            if table.has_identity_column() {
                self.dump_writer
                    .write_all(table.identity_insert_on().as_bytes())?;
                self.dump_writer.write_all(b"\r\n")?;
            }

            let mut count = 0;

            let mut reader = c.simple_query(query_from).await?;
            while let Some(Ok(row)) = reader.next().await {
                self.inc_progress_bar();

                if count % BATCH_SIZE == 0 {
                    if count > 0 {
                        self.dump_writer.write_all(b"\r\nGO\r\n")?;
                    }
                    self.dump_writer
                        .write_all(table.insert_statement().as_bytes())?;
                } else {
                    self.dump_writer.write_all(b",")?;
                }
                self.dump_writer.write_all(b"\r\n")?;

                let values: Vec<Value> = row.into_iter().map(|field| field.into()).collect();
                let values_str = if cfg.is_none() {
                    values
                        .into_iter()
                        .map(|v| v.into_dump_string())
                        .collect::<Vec<_>>()
                        .join(", ")
                } else {
                    self.engine
                        .process_row(
                            table.get_full_name(),
                            table.get_column_indexes(),
                            &values
                                .iter()
                                .map(|v| v.str.as_str())
                                .collect::<Vec<_>>()
                                .as_slice(),
                        )?
                        .iter()
                        .enumerate()
                        .map(|(i, v)| Value::dump_string(values[i].format, v))
                        .collect::<Vec<_>>()
                        .join(", ")
                };
                self.dump_writer
                    .write_all(format!("({})", values_str).as_bytes())?;

                count += 1;
            }
            self.dump_writer.write_all(b"\r\n")?;

            if table.has_identity_column() {
                self.dump_writer
                    .write_all(table.identity_insert_off().as_bytes())?;
                self.dump_writer.write_all(b"\r\n")?;
            }
            self.dump_writer.write_all(b"GO\r\n")?;

            self.finish_progress_bar();

            Ok(())
        })
    }
}

impl Dumper for MsSqlDumper {
    type Table = MsSqlTable;
    type Connection = Client<TcpStream>;
    type SchemaInspector = MsSqlSchemaInspector;

    fn pre_data(&mut self, _c: &mut Self::Connection) -> Result<()> {
        self.debug("Prepare data scheme...".into());

        if self.schema_dump.is_none() {
            self.load_schema_dump()?;
        }

        self.dump_writer
            .write_all(
                self.schema_dump
                    .as_ref()
                    .expect("missed schema dump")
                    .pre_data(),
            )
            .map_err(|e| e)
    }

    fn data(&mut self, c: &mut Self::Connection) -> Result<()> {
        self.debug("Dumping data...".into());

        let inspector = self.schema_inspector();
        self.write_log("DATA SECTION BEGIN".into())?;

        for table in inspector.get_tables(c)?.iter() {
            self.dump_table(c, table)?;
        }

        self.write_log("DATA SECTION END".into())?;

        Ok(())
    }

    fn post_data(&mut self, _c: &mut Self::Connection) -> Result<()> {
        self.debug("Finishing dump...".into());

        if self.schema_dump.is_none() {
            self.load_schema_dump()?;
        }

        self.dump_writer
            .write_all(
                self.schema_dump
                    .as_ref()
                    .expect("missed schema dump")
                    .post_data(),
            )
            .map_err(|e| e)
    }

    fn schema_inspector(&self) -> Self::SchemaInspector {
        self.schema_inspector.clone()
    }

    fn settings(&mut self) -> Settings {
        self.engine.settings.clone()
    }

    fn write_log(&mut self, message: String) -> Result<()> {
        self.dump_writer
            .write_all(format!("/****** {} ******/\r\n", message).as_bytes())
            .map_err(|e| e)
    }

    fn debug(&self, message: String) {
        if self.dump_writer.can_log_to_stdout() {
            println!("{}", message)
        }
    }
}

impl DumpProgressBar for MsSqlDumper {
    fn progress_bar(&self) -> &ProgressBar {
        &self.progress_bar
    }
}

#[cfg(test)]
mod tests {}
