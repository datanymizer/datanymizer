use super::{
    schema_inspector::MsSqlSchemaInspector, scripter::SchemaDump, table::MsSqlTable, value::Value,
};
use crate::{writer::DumpWriter, Dumper, SchemaInspector, Table};
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
        let inspector = self.schema_inspector();
        self.dump_writer
            .write_all(b"/****** DATA SECTION BEGIN ******/\n")?;

        for table in inspector.get_tables(c)?.iter() {
            self.dump_table(c, table)?;
        }

        self.dump_writer
            .write_all(b"/****** DATA SECTION END ******/\n")?;

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

    pub fn query(c: &mut <Self as Dumper>::Connection, q: &str) -> Result<Vec<Row>> {
        task::block_on(async { c.simple_query(q).await?.into_first_result().await })
            .map_err(|e| anyhow::Error::from(e))
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
        //for column in inspector.get_columns(connection, table)?.iter() {
        //     w.write_all(format!("{}\n", column.name).as_bytes())?;
        // }

        let settings = self.settings();
        self.write_log(format!("Dump table: {}", &table.get_full_name()))?;

        let cfg = settings.get_table(table.get_full_name().as_str());
        let query_from = table.query_from();

        task::block_on(async {
            self.dump_writer
                .write_all(table.identity_insert_on().as_bytes())?;
            self.dump_writer.write_all(b"\n")?;

            let mut count = 0;

            let mut reader = c.simple_query(query_from).await?;
            while let Some(Ok(row)) = reader.next().await {
                if count % BATCH_SIZE == 0 {
                    if count > 0 {
                        self.dump_writer.write_all(b"GO\n")?;
                    }
                    // INSERT .... VALUES
                    count = 0;
                }
                count += 1;

                let values: Vec<Value> = row.into_iter().map(|field| field.into()).collect();
                if cfg.is_none() {
                    // VALUES
                }
            }

            self.dump_writer.write_all(b"GO\n")?;
            self.dump_writer
                .write_all(table.identity_insert_off().as_bytes())?;
            self.dump_writer.write_all(b"\n")?;

            Ok(())
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
}
