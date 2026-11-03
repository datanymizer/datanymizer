use super::{quote_ident, schema_inspector::MySchemaInspector, Connection, MyTable};
use crate::{indicator::Indicator, Dumper, SchemaInspector, Table};
use anyhow::{anyhow, Result};
use datanymizer_engine::{
    AssertExpectation, AssertScope, AssertSeverity, Engine, Filter, Settings, TableList,
};
use log::warn;
use mysql::{consts::ColumnType, prelude::Queryable, AccessMode, Column, Opts, Row, TxOpts, Value};
use serde_json::Value as JsonValue;
use std::{borrow::Cow, io::prelude::*, process::Command, time::Instant};

pub use mysql::IsolationLevel;

/// Max size of one multi-row INSERT statement (must fit into `max_allowed_packet`)
const INSERT_BATCH_BYTES: usize = 1024 * 1024;

const DATA_HEADER: &str = "SET NAMES utf8mb4;
SET TIME_ZONE='+00:00';
SET FOREIGN_KEY_CHECKS=0;
SET UNIQUE_CHECKS=0;
SET SQL_MODE='NO_AUTO_VALUE_ON_ZERO';
";

const DATA_FOOTER: &str = "SET FOREIGN_KEY_CHECKS=1;
SET UNIQUE_CHECKS=1;
";

pub struct MyDumper<W: Write + Send, I: Indicator + Send> {
    schema_inspector: MySchemaInspector,
    engine: Engine,
    dump_writer: W,
    indicator: I,
    dump_isolation_level: Option<IsolationLevel>,
    mysqldump_location: String,
    mysqldump_args: Vec<String>,
    tables: Vec<MyTable>,
}

impl<W: 'static + Write + Send, I: 'static + Indicator + Send> MyDumper<W, I> {
    pub fn new(
        engine: Engine,
        dump_isolation_level: Option<IsolationLevel>,
        mysqldump_location: String,
        dump_writer: W,
        indicator: I,
        mysqldump_args: Vec<String>,
    ) -> Result<Self> {
        Ok(Self {
            schema_inspector: MySchemaInspector,
            engine,
            dump_writer,
            indicator,
            dump_isolation_level,
            mysqldump_location,
            mysqldump_args,
            tables: Vec::new(),
        })
    }

    fn run_mysqldump(&mut self, section_args: &[&str], opts: &Opts) -> Result<()> {
        let program = &self.mysqldump_location;
        let db = opts
            .get_db_name()
            .ok_or_else(|| anyhow!("No database name in the URL"))?;
        let mut args = vec![
            format!("--host={}", opts.get_ip_or_hostname()),
            format!("--port={}", opts.get_tcp_port()),
            // without it `localhost` means a socket and the port is ignored
            "--protocol=TCP".to_string(),
            "--no-tablespaces".to_string(),
        ];
        if let Some(user) = opts.get_user() {
            args.push(format!("--user={}", user));
        }
        args.extend(section_args.iter().map(|s| s.to_string()));
        args.extend(self.mysqldump_args.iter().cloned());
        args.extend(table_args(&self.engine.settings.filter, db));

        let mut cmd = Command::new(program);
        // ponytail: MYSQL_PWD is deprecated by MySQL but still honored; switch to
        // --defaults-extra-file if a client version drops it
        if let Some(pass) = opts.get_pass() {
            cmd.env("MYSQL_PWD", pass);
        }
        let dump_output = cmd.args(&args).output()?;
        if !dump_output.status.success() {
            return Err(anyhow!(
                "mysqldump error. Command:\n{} {}\nOutput:\n{}",
                program,
                args.join(" "),
                String::from_utf8_lossy(&dump_output.stderr)
            ));
        }

        self.dump_writer
            .write_all(&dump_output.stdout)
            .map_err(|e| e.into())
    }

    fn dump_table<Q: Queryable>(&mut self, table: &MyTable, q: &mut Q) -> Result<()> {
        let started = Instant::now();
        self.write_log(format!("Dump table: {}", table.get_full_name()))?;

        let cfg = self.engine.settings.find_table(&table.get_names());
        self.indicator
            .start_pb(table.count_of_query_to(cfg), &table.get_full_name());

        // like mysqldump: faster restore of big tables
        let name = quote_ident(&table.tablename);
        self.dump_writer.write_all(
            format!("LOCK TABLES {name} WRITE;\n/*!40000 ALTER TABLE {name} DISABLE KEYS */;\n")
                .as_bytes(),
        )?;

        let mut writer = InsertWriter::new(table);
        let mut count: u64 = 0;
        if let Some(cfg) = cfg {
            if let Some(query) = table.transformed_query_to(Some(cfg), count) {
                for row in q.query_iter(query)? {
                    let row = row?;
                    self.indicator.inc_pb(1);

                    let (texts, literals) = row_cells(&row);
                    let values: Vec<&str> = texts.iter().map(|s| s.as_str()).collect();
                    let transformed = self
                        .engine
                        .process_row(cfg.name.clone(), table.get_column_indexes(), &values)
                        .map_err(|err| {
                            warn!("{:#?}", err);
                            err
                        })?;
                    let literals = transformed
                        .into_iter()
                        .zip(literals)
                        .map(|(v, lit)| match v {
                            Cow::Borrowed(_) => lit,
                            Cow::Owned(s) if s == NULL_TEXT => "NULL".to_string(),
                            Cow::Owned(s) => quote_str(&s),
                        });
                    writer.write_row(&mut self.dump_writer, literals)?;
                    count += 1;
                }
            }
        }

        if let Some(query) = table.untransformed_query_to(cfg, count) {
            for row in q.query_iter(query)? {
                let row = row?;
                self.indicator.inc_pb(1);
                writer.write_row(&mut self.dump_writer, row_cells(&row).1.into_iter())?;
            }
        }
        writer.finish(&mut self.dump_writer)?;
        self.dump_writer.write_all(
            format!("/*!40000 ALTER TABLE {name} ENABLE KEYS */;\nUNLOCK TABLES;\n").as_bytes(),
        )?;

        self.indicator
            .finish_pb(table.get_full_name().as_str(), started.elapsed());
        Ok(())
    }

    fn dump_tables<Q: Queryable>(&mut self, q: &mut Q) -> Result<()> {
        q.query_drop("SET TIME_ZONE='+00:00'")?;
        let all_tables_count = self.tables.len();
        for (ind, table) in self.tables.clone().iter().enumerate() {
            self.debug(format!(
                "[{} / {}] Prepare to dump table: {}",
                ind + 1,
                all_tables_count,
                table.get_full_name(),
            ));
            if self.filter_table(table.get_full_name()) {
                self.dump_table(table, q)?;
            } else {
                self.debug(format!("[Dumping: {}] --- SKIP ---", table.get_full_name()));
            }
        }
        Ok(())
    }

    fn tx_opts(level: IsolationLevel) -> TxOpts {
        TxOpts::default()
            .set_isolation_level(Some(level))
            .set_with_consistent_snapshot(true)
            .set_access_mode(Some(AccessMode::ReadOnly))
    }
}

impl<W: 'static + Write + Send, I: 'static + Indicator + Send> Dumper for MyDumper<W, I> {
    type Connection = Connection;
    type SchemaInspector = MySchemaInspector;

    fn pre_data(&mut self, connection: &mut Self::Connection) -> Result<()> {
        self.debug("Prepare data scheme...".into());
        match self.dump_isolation_level {
            Some(level) => {
                let mut tx = connection.conn.start_transaction(Self::tx_opts(level))?;
                run_asserts(&self.engine.settings, &mut tx)?;
            }
            None => run_asserts(&self.engine.settings, &mut connection.conn)?,
        }
        let opts = connection.opts.clone();
        // triggers go to the post-data stage, otherwise they would fire on data load
        self.run_mysqldump(&["--no-data", "--skip-triggers"], &opts)
    }

    fn data(&mut self, connection: &mut Self::Connection) -> Result<()> {
        self.write_log("Start dumping data".into())?;
        self.dump_writer.write_all(DATA_HEADER.as_bytes())?;
        match self.dump_isolation_level {
            Some(level) => {
                let mut tx = connection.conn.start_transaction(Self::tx_opts(level))?;
                self.dump_tables(&mut tx)?;
                tx.commit()?;
            }
            None => self.dump_tables(&mut connection.conn)?,
        }
        self.dump_writer.write_all(DATA_FOOTER.as_bytes())?;
        self.write_log("End dumping data".into())?;
        Ok(())
    }

    fn post_data(&mut self, connection: &mut Self::Connection) -> Result<()> {
        self.debug("Finishing with triggers...".into());
        let opts = connection.opts.clone();
        self.run_mysqldump(&["--no-data", "--no-create-info", "--triggers"], &opts)
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
            .write_all(format!("\n--\n-- {}\n--\n", message).as_bytes())
            .map_err(|e| e.into())
    }

    fn debug(&self, message: String) {
        self.indicator.debug_msg(message.as_str());
    }
}

/// mysqldump takes tables as positional args (only) or `--ignore-table=db.table` (except)
fn table_args(f: &Filter, db: &str) -> Vec<String> {
    let prefix = format!("{}.", db);
    let list = f.schema_match_list();
    let mut args = vec![db.to_string()];
    for table in list.tables() {
        let name = table.strip_prefix(&prefix).unwrap_or(table);
        args.push(match list {
            TableList::Only(_) => name.to_string(),
            TableList::Except(_) => format!("--ignore-table={}.{}", db, name),
        });
    }
    args
}

/// Same NULL marker as in Postgres dumps, so configs behave equally for both databases
const NULL_TEXT: &str = "\\N";

/// Returns per-cell text for the transform engine and SQL literals for untransformed output
fn row_cells(row: &Row) -> (Vec<String>, Vec<String>) {
    row.columns_ref()
        .iter()
        .enumerate()
        .map(|(i, column)| (row.as_ref(i).unwrap_or(&Value::NULL), column))
        .map(|(value, column)| match value {
            Value::NULL => (NULL_TEXT.to_string(), "NULL".to_string()),
            Value::Bytes(b) if is_binary(column) => {
                let hex = if b.is_empty() {
                    "''".to_string()
                } else {
                    format!(
                        "0x{}",
                        b.iter().map(|x| format!("{:02X}", x)).collect::<String>()
                    )
                };
                (hex.clone(), hex)
            }
            Value::Bytes(b) => {
                let s = String::from_utf8_lossy(b).into_owned();
                let lit = quote_str(&s);
                (s, lit)
            }
            // the text protocol returns only NULL or bytes, but keep a sane fallback
            v => (v.as_sql(true), v.as_sql(true)),
        })
        .unzip()
}

fn is_binary(column: &Column) -> bool {
    use ColumnType::*;
    // numbers and dates also report the binary charset (63), so the type matters too
    column.character_set() == 63
        && matches!(
            column.column_type(),
            MYSQL_TYPE_STRING
                | MYSQL_TYPE_VAR_STRING
                | MYSQL_TYPE_VARCHAR
                | MYSQL_TYPE_BLOB
                | MYSQL_TYPE_TINY_BLOB
                | MYSQL_TYPE_MEDIUM_BLOB
                | MYSQL_TYPE_LONG_BLOB
                | MYSQL_TYPE_BIT
                | MYSQL_TYPE_GEOMETRY
        )
}

fn quote_str(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('\'');
    for c in s.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '\'' => out.push_str("\\'"),
            '\0' => out.push_str("\\0"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\x1a' => out.push_str("\\Z"),
            c => out.push(c),
        }
    }
    out.push('\'');
    out
}

/// Writes rows as multi-row INSERT statements
struct InsertWriter {
    prefix: String,
    batch_bytes: usize,
}

impl InsertWriter {
    fn new(table: &MyTable) -> Self {
        Self {
            // unqualified like mysqldump's DDL, so the dump can be restored into any database
            prefix: format!(
                "INSERT INTO {} ({}) VALUES\n",
                quote_ident(&table.tablename),
                table.quoted_columns()
            ),
            batch_bytes: 0,
        }
    }

    fn write_row<W: Write>(
        &mut self,
        w: &mut W,
        literals: impl Iterator<Item = String>,
    ) -> Result<()> {
        let row = format!("({})", literals.collect::<Vec<_>>().join(","));
        if self.batch_bytes == 0 {
            w.write_all(self.prefix.as_bytes())?;
        } else {
            w.write_all(b",\n")?;
        }
        w.write_all(row.as_bytes())?;
        self.batch_bytes += row.len();
        if self.batch_bytes >= INSERT_BATCH_BYTES {
            self.finish(w)?;
        }
        Ok(())
    }

    fn finish<W: Write>(&mut self, w: &mut W) -> Result<()> {
        if self.batch_bytes > 0 {
            w.write_all(b";\n")?;
            self.batch_bytes = 0;
        }
        Ok(())
    }
}

/// Executes configured SQL assertions before dump generation starts.
fn run_asserts<Q: Queryable>(settings: &Settings, q: &mut Q) -> Result<()> {
    let mut warnings = Vec::new();
    for scope in settings.all_asserts() {
        if let Err(err) = run_assert(scope, q) {
            match scope.assert().severity {
                AssertSeverity::Error => return Err(err),
                AssertSeverity::Warn => warnings.push(err.to_string()),
            }
        }
    }
    if !warnings.is_empty() {
        eprintln!("Assertions warnings:");
        for warning in warnings {
            eprintln!("- {}", warning);
        }
    }
    Ok(())
}

fn run_assert<Q: Queryable>(scope: AssertScope<'_>, q: &mut Q) -> Result<()> {
    let assert = scope.assert();
    let exists_query = || {
        format!(
            "SELECT EXISTS(SELECT 1 FROM ({}) AS datanymizer_assert LIMIT 1)",
            assert.sql
        )
    };
    match &assert.expect {
        AssertExpectation::NoRows => {
            let has_rows: Option<i64> = q.query_first(exists_query())?;
            scope.expect_no_rows(has_rows.unwrap_or_default() != 0)?;
        }
        AssertExpectation::RowsExist => {
            let has_rows: Option<i64> = q.query_first(exists_query())?;
            scope.expect_rows_exist(has_rows.unwrap_or_default() != 0)?;
        }
        AssertExpectation::Scalar(_) => {
            let row: Row = q
                .query_first(format!(
                    "SELECT * FROM ({}) AS datanymizer_assert",
                    assert.sql
                ))?
                .ok_or_else(|| anyhow!(scope.missing_scalar_value()))?;
            let actual = match (row.as_ref(0), row.columns_ref().first()) {
                (Some(Value::Bytes(b)), Some(col)) => {
                    let s = String::from_utf8_lossy(b);
                    if is_numeric(col) {
                        serde_json::from_str(&s).unwrap_or(JsonValue::String(s.into_owned()))
                    } else {
                        JsonValue::String(s.into_owned())
                    }
                }
                _ => JsonValue::Null,
            };
            scope.expect_scalar(&actual)?;
        }
    }
    Ok(())
}

fn is_numeric(column: &Column) -> bool {
    use ColumnType::*;
    matches!(
        column.column_type(),
        MYSQL_TYPE_TINY
            | MYSQL_TYPE_SHORT
            | MYSQL_TYPE_INT24
            | MYSQL_TYPE_LONG
            | MYSQL_TYPE_LONGLONG
            | MYSQL_TYPE_FLOAT
            | MYSQL_TYPE_DOUBLE
            | MYSQL_TYPE_DECIMAL
            | MYSQL_TYPE_NEWDECIMAL
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mysql::MyColumn;

    #[test]
    fn quoting() {
        assert_eq!(quote_str("a'b\\c\nd\0\r\x1a"), r"'a\'b\\c\nd\0\r\Z'");
        assert_eq!(quote_ident("we`ird"), "`we``ird`");
    }

    #[test]
    fn queries() {
        let mut t = MyTable::new("users".into(), "app".into());
        t.set_columns(vec![MyColumn {
            position: 1,
            name: "id".into(),
            data_type: "int".into(),
        }]);
        assert_eq!(
            t.query_with_select(vec![Some("(id > 1)".into())], Some(5)),
            "SELECT `id` FROM `app`.`users` WHERE (id > 1) LIMIT 5"
        );
    }

    #[test]
    fn insert_batches() {
        let t = MyTable::new("t".into(), "db".into());
        let mut w = InsertWriter::new(&t);
        let mut out = Vec::new();
        w.write_row(
            &mut out,
            vec!["1".to_string(), "NULL".to_string()].into_iter(),
        )
        .unwrap();
        w.write_row(
            &mut out,
            vec!["2".to_string(), "'x'".to_string()].into_iter(),
        )
        .unwrap();
        w.finish(&mut out).unwrap();
        w.finish(&mut out).unwrap();
        assert_eq!(
            String::from_utf8(out).unwrap(),
            "INSERT INTO `t` () VALUES\n(1,NULL),\n(2,'x');\n"
        );
    }
}
