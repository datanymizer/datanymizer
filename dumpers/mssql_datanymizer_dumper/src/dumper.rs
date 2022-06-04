use super::{
    connector, schema_inspector::MsSqlSchemaInspector, scripter::SchemaDump, table::MsSqlTable,
    value::Value,
};
use anyhow::Result;
use async_std::{stream::StreamExt, task};
use datanymizer_dumper::{indicator::Indicator, Dumper, SchemaInspector, Table};
use datanymizer_engine::{Engine, Filter, Settings, TableList};
use std::{io::Write, process::Command, time::Instant};

// https://docs.microsoft.com/en-us/sql/t-sql/queries/table-value-constructor-transact-sql?view=sql-server-ver15#limitations-and-restrictions
const BATCH_SIZE: usize = 1000;

pub struct MsSqlDumper<W: Write + Send, I: Indicator + Send> {
    schema_inspector: MsSqlSchemaInspector,
    engine: Engine,
    dump_writer: W,
    indicator: I,
    mssql_scripter_location: String,
    schema_dump: Option<SchemaDump>,
    tables: Vec<MsSqlTable>,
}

impl<W: 'static + Write + Send, I: 'static + Indicator + Send> MsSqlDumper<W, I> {
    pub fn new(
        engine: Engine,
        mssql_scripter_location: String,
        dump_writer: W,
        indicator: I,
    ) -> Result<Self> {
        Ok(Self {
            schema_inspector: MsSqlSchemaInspector,
            engine,
            dump_writer,
            indicator,
            mssql_scripter_location,
            schema_dump: None,
            tables: Vec::new(),
        })
    }

    fn load_schema_dump(&mut self, connection: &mut <Self as Dumper>::Connection) -> Result<()> {
        let dump_output = Command::new(&self.mssql_scripter_location)
            .args(table_args(&self.engine.settings.filter))
            .args(&["--connection-string", connection.url.as_str()])
            .output()?;

        self.schema_dump = Some(SchemaDump::new(dump_output.stdout));

        Ok(())
    }

    fn dump_table(
        &mut self,
        c: &mut <Self as Dumper>::Connection,
        table: &MsSqlTable,
    ) -> Result<()> {
        let started = Instant::now();
        self.indicator
            .start_pb(table.get_size() as u64, &table.get_full_name());

        self.write_log(format!("Data for {}", table.get_full_name()))?;
        self.dump_writer
            .write_all(format!("PRINT N'Restoring {}...'\r\n", table.get_full_name()).as_bytes())?;

        let query_from = table.query_from();

        task::block_on(async {
            let settings = self.settings().clone();
            let cfg = settings.get_table(table.get_full_name().as_str());
            if table.has_identity_column() {
                self.dump_writer
                    .write_all(table.identity_insert_on().as_bytes())?;
                self.dump_writer.write_all(b"\r\n")?;
            }

            let mut count = 0;

            let mut reader = c.client.simple_query(query_from).await?;
            while let Some(Ok(row)) = reader.next().await {
                self.indicator.inc_pb(1);

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
                            values
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

            let finished = started.elapsed();
            self.indicator
                .finish_pb(table.get_full_name().as_str(), finished);
            Ok(())
        })
    }
}

impl<W: 'static + Write + Send, I: 'static + Indicator + Send> Dumper for MsSqlDumper<W, I> {
    type Connection = connector::Connection;
    type SchemaInspector = MsSqlSchemaInspector;

    fn pre_data(&mut self, c: &mut Self::Connection) -> Result<()> {
        self.debug("Prepare data scheme...".into());

        if self.schema_dump.is_none() {
            self.load_schema_dump(c)?;
        }

        self.dump_writer
            .write_all(
                self.schema_dump
                    .as_ref()
                    .expect("missed schema dump")
                    .pre_data(),
            )
            .map_err(|e| e.into())
    }

    fn data(&mut self, c: &mut Self::Connection) -> Result<()> {
        self.debug("Dumping data...".into());

        let inspector = self.schema_inspector();

        self.write_log("DATA SECTION BEGIN".into())?;

        for table in inspector.get_tables(c)?.iter() {
            if self.filter_table(table.get_full_name()) {
                self.dump_table(c, table)?;
            } else {
                self.debug(format!("[Dumping: {}] --- SKIP ---", table.get_full_name()));
            }
        }

        self.write_log("DATA SECTION END".into())?;

        Ok(())
    }

    fn post_data(&mut self, c: &mut Self::Connection) -> Result<()> {
        self.debug("Finishing dump...".into());

        if self.schema_dump.is_none() {
            self.load_schema_dump(c)?;
        }

        self.dump_writer
            .write_all(
                self.schema_dump
                    .as_ref()
                    .expect("missed schema dump")
                    .post_data(),
            )
            .map_err(|e| e.into())
    }

    fn schema_inspector(&self) -> Self::SchemaInspector {
        self.schema_inspector.clone()
    }

    fn set_tables(&mut self, tables: Vec<<Self::SchemaInspector as SchemaInspector>::Table>) {
        self.tables = tables;
    }

    fn settings(&self) -> &Settings {
        &self.engine.settings
    }

    fn filter_mut(&mut self) -> &mut Filter {
        &mut self.engine.settings.filter
    }

    fn write_log(&mut self, message: String) -> Result<()> {
        self.dump_writer
            .write_all(format!("/****** {} ******/\r\n", message).as_bytes())
            .map_err(|e| e.into())
    }

    fn debug(&self, message: String) {
        self.indicator.debug_msg(message.as_str());
    }
}

fn table_args(f: &Filter) -> Vec<String> {
    let mut args = vec![];

    let list = f.schema_match_list();
    let flag = match list {
        TableList::Only(_) => "--include-objects",
        TableList::Except(_) => "--exclude-objects",
    };
    for table in list.tables() {
        args.push(String::from(flag));
        args.push(table.clone());
    }

    args
}
