use super::{schema_inspector::MsSqlSchemaInspector, scripter::SchemaDump, table::MsSqlTable};
use crate::{writer::DumpWriter, Dumper, SchemaInspector, Table};
use anyhow::Result;
use async_std::net::TcpStream;
use datanymizer_engine::{Engine, Settings};
use indicatif::ProgressBar;
use std::process::Command;
use tiberius::Client;

pub struct MsSqlDumper {
    schema_inspector: MsSqlSchemaInspector,
    engine: Engine,
    dump_writer: DumpWriter,
    mssql_scripter_location: String,
    progress_bar: ProgressBar,
    schema_dump: Option<SchemaDump>,
}

impl Dumper for MsSqlDumper {
    type Table = MsSqlTable;
    type Connection = Client<TcpStream>;
    type SchemaInspector = MsSqlSchemaInspector;

    fn pre_data(&mut self, _connection: &mut Self::Connection) -> Result<()> {
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

    fn data(&mut self, connection: &mut Self::Connection) -> Result<()> {
        let inspector = self.schema_inspector();
        let w = &mut self.dump_writer;
        w.write_all(b"/************** DATA BEGIN ****************/\n")?;

        for table in inspector.get_tables(connection)?.iter() {
            w.write_all(table.get_full_name().as_bytes())?;
            w.write_all(b"\n")?;
        }

        self.dump_writer
            .write_all(b"/************** DATA END ****************/\n")?;

        Ok(())
    }

    fn post_data(&mut self, _connection: &mut Self::Connection) -> Result<()> {
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
            .write_all(format!("\n---\n--- {}\n---\n", message).as_bytes())
            .map_err(|e| e)
    }

    fn debug(&self, message: String) {
        if self.dump_writer.can_log_to_stdout() {
            println!("{}", message)
        }
    }
}

impl MsSqlDumper {
    pub fn new(
        engine: Engine,
        mssql_scripter_location: String,
        target: Option<String>,
    ) -> Result<Self> {
        let dump_writer = DumpWriter::new(target)?;
        let pb: ProgressBar = if dump_writer.can_log_to_stdout() {
            ProgressBar::new(0)
        } else {
            ProgressBar::hidden()
        };
        Ok(Self {
            engine,
            dump_writer,
            mssql_scripter_location,
            schema_inspector: MsSqlSchemaInspector,
            progress_bar: pb,
            schema_dump: None,
        })
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
}

#[cfg(test)]
mod tests {
    use super::*;
}
