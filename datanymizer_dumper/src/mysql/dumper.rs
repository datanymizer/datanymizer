use super::{
    connector::Connection, decoder::Decoder, schema_inspector::MysqlSchemaInspector,
    table::MysqlTable,
};
use crate::{indicator::Indicator, Dumper, SchemaInspector, Table};
use anyhow::Result;
use datanymizer_engine::{Engine, Filter, Settings, TableList};
use futures::stream::TryStreamExt;
use log::warn;
use sqlx::{mysql::MySqlRow, Column, Row};
use std::{
    borrow::Cow,
    collections::HashMap,
    io::{self, prelude::*},
    process::{self, Command},
    time::Instant,
};

const L_TABLE_NAME_MARKER: &[u8] = "\n--\n-- Table structure for table `".as_bytes();
const R_TABLE_NAME_MARKER: u8 = "\n".as_bytes()[0];
const END_OF_TABLE_MARKER: &[u8] =
    "/*!40101 SET character_set_client = @saved_cs_client */;\n".as_bytes();
const BATCH_SIZE: usize = 1000;

pub struct MysqlDumper<W: Write + Send, I: Indicator + Send> {
    schema_inspector: MysqlSchemaInspector,
    engine: Engine,
    dump_writer: W,
    indicator: I,
    mysqldump_location: String,
    mysqldump_args: Vec<String>,
    tables: Vec<MysqlTable>,
    schema_data: Vec<u8>,
    data_offsets: HashMap<String, (usize, usize, usize)>,
    count: u64,
}

impl<W: 'static + Write + Send, I: 'static + Indicator + Send> MysqlDumper<W, I> {
    pub fn new(
        engine: Engine,
        mysqldump_location: String,
        dump_writer: W,
        indicator: I,
        mysqldump_args: Vec<String>,
    ) -> Result<Self> {
        Ok(Self {
            engine,
            dump_writer,
            indicator,
            mysqldump_location,
            schema_inspector: MysqlSchemaInspector {},
            mysqldump_args,
            tables: Vec::new(),
            schema_data: vec![],
            data_offsets: HashMap::new(),
            count: 0,
        })
    }

    fn run_mysqldump(&mut self, conn: &Connection) -> Result<Vec<u8>> {
        let db_url = &conn.url;
        let program = &self.mysqldump_location;
        let mut args = vec!["--no-data"];
        if let Some(host) = db_url.host_str() {
            args.push("-h");
            args.push(host);
        }
        let port_str: String;
        if let Some(port) = db_url.port() {
            port_str = port.to_string();
            args.push("-P");
            args.push(&port_str);
        }
        let username = db_url.username();
        if !username.is_empty() {
            args.push("-u");
            args.push(username);
        }
        let pass_str: String;
        if let Some(password) = db_url.password() {
            pass_str = format!("--password={}", password);
            args.push(&pass_str);
        }
        let db_name = conn.db_name();
        args.push(db_name);

        let table_args = table_args(&self.engine.settings.filter, db_name);
        let dump_output = Command::new(program)
            .args(&self.mysqldump_args)
            .args(&args)
            .args(&table_args)
            .output()?;
        if !dump_output.status.success() {
            eprintln!(
                "mysqldump error. Command:\n{} {}\nOutput:",
                program,
                args.into_iter()
                    .chain(table_args.iter().map(|s| s.as_str()))
                    .collect::<Vec<_>>()
                    .join(" ")
            );

            io::stderr().write_all(&dump_output.stderr)?;
            process::exit(1);
        }

        Ok(dump_output.stdout)
    }

    fn fill_data_offsets(&mut self) -> Result<()> {
        let mut prev_table: Option<String> = None;
        for (i, _) in self
            .schema_data
            .windows(L_TABLE_NAME_MARKER.len())
            .enumerate()
            .filter(|(_, w)| (*w).eq(L_TABLE_NAME_MARKER))
        {
            let curr_offset = i + L_TABLE_NAME_MARKER.len();
            let rest = &self.schema_data[curr_offset..];
            let end_of_name = rest
                .iter()
                .position(|i| *i == R_TABLE_NAME_MARKER)
                .expect("Dump format error (table name)")
                - 1;
            let table_name = String::from_utf8(rest[..end_of_name].to_vec())?;
            let end_of_tbl_schema = rest
                .windows(END_OF_TABLE_MARKER.len())
                .position(|w| w.eq(END_OF_TABLE_MARKER))
                .expect("Dump format error")
                + curr_offset
                + END_OF_TABLE_MARKER.len();
            self.data_offsets.insert(
                table_name.clone(),
                (i, end_of_tbl_schema, end_of_tbl_schema),
            );
            if let Some(prev_table) = prev_table.as_ref() {
                let (start, finish, _) = self.data_offsets[prev_table];
                self.data_offsets
                    .insert(prev_table.clone(), (start, finish, i));
            }
            prev_table = Some(table_name);
        }

        Ok(())
    }

    fn dump_table(&mut self, table: &MysqlTable, connection: &mut Connection) -> Result<()> {
        let started = Instant::now();
        let table_name = table.get_name();

        self.dump_writer.write_all(
            format!("\n--\n-- Dumping data for table `{}`\n--\n", table_name).as_bytes(),
        )?;
        self.dump_writer.write_all(
            format!(
                "LOCK TABLES `{}` WRITE;\n/*!40000 ALTER TABLE `{}` DISABLE KEYS */;\n",
                table_name, table_name
            )
            .as_bytes(),
        )?;

        let cfg = self.engine.settings.find_table(&table.get_names()).cloned();

        self.indicator
            .start_pb(table.count_of_query_to(cfg.as_ref()), &table_name);

        self.count = 0;
        if let Some(cfg) = cfg.clone() {
            if let Some(transformed_query) = table.transformed_query_to(Some(&cfg), self.count) {
                self.dump_rows(
                    table,
                    connection,
                    transformed_query.as_str(),
                    |dumper, table, decoders, row| {
                        dumper
                            .transform(table, row, decoders)
                            .map_err(|err| {
                                warn!("{:#?}", err);
                                err
                            })
                            .expect("Transformation error")
                    },
                )?;
            }
        }

        if let Some(untransformed_query) = table.untransformed_query_to(cfg.as_ref(), self.count) {
            self.dump_rows(
                table,
                connection,
                untransformed_query.as_str(),
                |_dumper, _table, decoders, row| Self::decode_row(row, decoders, true).join(","),
            )?;
        }

        self.dump_writer.write_all(
            format!(
                "/*!40000 ALTER TABLE `{}` ENABLE KEYS */;\nUNLOCK TABLES;\n",
                table_name
            )
            .as_bytes(),
        )?;

        let finished = started.elapsed();
        self.indicator
            .finish_pb(table.get_name().as_str(), finished);

        Ok(())
    }

    fn dump_rows(
        &mut self,
        table: &MysqlTable,
        connection: &mut Connection,
        query: &str,
        format_row: fn(
            dumper: &Self,
            table: &MysqlTable,
            decoders: &[Decoder],
            row: &MySqlRow,
        ) -> String,
    ) -> Result<()> {
        connection.rt.block_on(async {
            let mut reader = sqlx::query(query).fetch(&mut connection.conn);
            let mut batch = Vec::with_capacity(BATCH_SIZE);
            let mut decoders: Option<Vec<Decoder>> = None;
            loop {
                let row = reader.try_next().await?;
                if let Some(row) = row {
                    self.count += 1;
                    self.indicator.inc_pb(1);

                    if decoders.is_none() {
                        decoders = Some(
                            row.columns()
                                .iter()
                                .map(|col| col.type_info().try_into().expect("Unsupported DB type"))
                                .collect(),
                        );
                    }

                    batch.push(row);
                    if batch.len() == BATCH_SIZE {
                        self.dump_writer.write_all(
                            self.insert_query(
                                table,
                                decoders.as_ref().expect("Decoders are not ready"),
                                batch.as_slice(),
                                format_row,
                            )
                            .as_bytes(),
                        )?;
                        batch.clear();
                    }
                } else {
                    if !batch.is_empty() {
                        self.dump_writer.write_all(
                            self.insert_query(
                                table,
                                decoders.as_ref().expect("Decoders are not ready"),
                                batch.as_slice(),
                                format_row,
                            )
                            .as_bytes(),
                        )?;
                    }
                    return Ok::<(), anyhow::Error>(());
                }
            }
        })
    }

    fn insert_query(
        &self,
        table: &MysqlTable,
        decoders: &[Decoder],
        rows: &[MySqlRow],
        format_row: fn(
            dumper: &Self,
            table: &MysqlTable,
            decoders: &[Decoder],
            row: &MySqlRow,
        ) -> String,
    ) -> String {
        let mut query = format!("INSERT INTO `{}` VALUES ", table.get_name());
        let len = rows.len();
        for (i, row) in rows.iter().enumerate() {
            query.push_str(format!("({})", format_row(self, table, decoders, row)).as_str());
            if i < len - 1 {
                query.push(',');
            }
        }
        query.push_str(";\n");
        query
    }

    fn decode_row(row: &MySqlRow, decoders: &[Decoder], need_quote: bool) -> Vec<String> {
        (0..row.len())
            .map(|i| {
                let decoder = &decoders[i];
                let value = row.try_get_raw(i).expect("Error while fetch data");
                if need_quote {
                    decoder.decode_and_quote(value)
                } else {
                    decoder.decode(value)
                }
                .expect("Error while decoding data")
            })
            .collect::<Vec<_>>()
    }

    fn transform(&self, tbl: &MysqlTable, row: &MySqlRow, decoders: &[Decoder]) -> Result<String> {
        let values = Self::decode_row(row, decoders, false);
        let ref_values = values.iter().map(|s| s.as_str()).collect::<Vec<_>>();
        let mut transformed_values =
            self.engine
                .process_row(tbl.get_name(), tbl.get_column_indexes(), &ref_values)?;
        for (i, v) in &mut transformed_values.iter_mut().enumerate() {
            let res = decoders[i].prepare_output(&v);
            if let Some(res) = res {
                *v = Cow::Owned(res);
            }
        }

        Ok(transformed_values.join(","))
    }
}

impl<W: 'static + Write + Send, I: 'static + Indicator + Send> Dumper for MysqlDumper<W, I> {
    type Connection = Connection;
    type SchemaInspector = MysqlSchemaInspector;

    fn pre_data(&mut self, connection: &mut Self::Connection) -> Result<()> {
        self.debug("Prepare data scheme...".into());
        self.schema_data = self.run_mysqldump(connection)?;
        self.fill_data_offsets()?;

        let first_table = self.tables.first();
        if let Some(first_table) = first_table {
            let first_offset = self.data_offsets[&first_table.get_name()].0;
            self.dump_writer
                .write_all(&self.schema_data[..first_offset])
                .map_err(|e| e.into())
        } else {
            self.dump_writer
                .write_all(&self.schema_data)
                .map_err(|e| e.into())
        }
    }

    fn data(&mut self, connection: &mut Self::Connection) -> Result<()> {
        let all_tables_count = self.tables.len();

        for (ind, table) in self.tables.clone().iter().enumerate() {
            let table_name = table.get_name();
            self.debug(format!(
                "[{} / {}] Prepare to dump table: {}",
                ind + 1,
                all_tables_count,
                &table_name,
            ));

            if self.data_offsets.contains_key(&table_name) {
                let (start, finish, suffix) = self.data_offsets[&table_name];
                self.dump_writer
                    .write_all(&self.schema_data[start..finish])?;
                if self.filter_table(table_name.clone()) {
                    self.dump_table(table, connection)?;
                } else {
                    self.debug(format!("[Dumping: {}] --- SKIP DATA ---", table_name));
                }
                self.dump_writer
                    .write_all(&self.schema_data[finish..suffix])?;
            } else {
                self.debug(format!("[Dumping: {}] --- SKIP SCHEMA ---", table_name));
            }
        }

        Ok(())
    }

    fn post_data(&mut self, _connection: &mut Self::Connection) -> Result<()> {
        self.debug("Finishing...".into());

        let last_table = self.tables.last();
        if let Some(last_table) = last_table {
            let last_offset = self.data_offsets[&last_table.get_name()].1;
            self.dump_writer
                .write_all(&self.schema_data[last_offset..])
                .map_err(|e| e.into())
        } else {
            Ok(())
        }
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
            .write_all(format!("\n---\n--- {}\n---\n", message).as_bytes())
            .map_err(|e| e.into())
    }

    fn debug(&self, message: String) {
        self.indicator.debug_msg(message.as_str());
    }
}

fn table_args(f: &Filter, db_name: &str) -> Vec<String> {
    let list = f.schema_match_list();
    match list {
        TableList::Only(tables) => tables.clone(),
        TableList::Except(tables) => tables
            .iter()
            .map(|s| format!("--ignore-table={}.{}", db_name, s))
            .collect(),
    }
}
