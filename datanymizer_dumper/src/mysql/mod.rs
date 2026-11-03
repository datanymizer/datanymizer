use crate::{sql_conditions, sql_limit, ColumnData, Table};
use anyhow::Result;
use mysql::{consts::ColumnType, Conn, Opts, OptsBuilder, SslOpts};
use std::{
    collections::HashMap,
    hash::{Hash, Hasher},
};
use url::Url;

pub mod dumper;
pub mod schema_inspector;

pub struct Connection {
    pub conn: Conn,
    pub opts: Opts,
}

pub struct Connector {
    url: Url,
    accept_invalid_hostnames: bool,
    accept_invalid_certs: bool,
}

impl Connector {
    pub fn new(url: Url, accept_invalid_hostnames: bool, accept_invalid_certs: bool) -> Self {
        Self {
            url,
            accept_invalid_hostnames,
            accept_invalid_certs,
        }
    }

    /// TLS is configured via URL params of the `mysql` crate (e.g. `?require_ssl=true`)
    pub fn connect(&self) -> Result<Connection> {
        let opts = Opts::from_url(self.url.as_str())?;
        let ssl_opts = opts.get_ssl_opts().cloned().map(|o: SslOpts| {
            o.with_danger_skip_domain_validation(self.accept_invalid_hostnames)
                .with_danger_accept_invalid_certs(self.accept_invalid_certs)
        });
        let opts: Opts = OptsBuilder::from_opts(opts).ssl_opts(ssl_opts).into();
        let mut conn = Conn::new(opts.clone())?;
        mysql::prelude::Queryable::query_drop(&mut conn, "SET NAMES utf8mb4")?;

        Ok(Connection { conn, opts })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MyColumn {
    pub position: usize,
    pub name: String,
    pub data_type: String,
}

impl ColumnData<ColumnType> for MyColumn {
    fn position(&self) -> usize {
        self.position - 1
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn inner_kind(&self) -> Option<ColumnType> {
        None
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MyForeignKey {
    pub constraint_name: String,
    pub column_name: String,
    pub foreign_table_schema: String,
    pub foreign_table_name: String,
    pub foreign_column_name: String,
}

#[derive(Debug, Clone, Eq)]
pub struct MyTable {
    pub tablename: String,
    pub schemaname: String,
    pub columns: Vec<MyColumn>,
    pub foreign_keys: Vec<MyForeignKey>,
    column_indexes: HashMap<String, usize>,
    pub size: i64,
}

impl PartialEq for MyTable {
    fn eq(&self, other: &Self) -> bool {
        self.get_full_name() == other.get_full_name()
    }
}

impl Hash for MyTable {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.tablename.hash(state);
        self.schemaname.hash(state);
    }
}

impl Table<ColumnType> for MyTable {
    type Column = MyColumn;
    type Row = ();

    fn get_name(&self) -> String {
        self.tablename.clone()
    }

    fn get_full_name(&self) -> String {
        format!("{}.{}", self.schemaname, self.tablename)
    }

    fn get_names(&self) -> Vec<String> {
        vec![self.get_full_name(), self.get_name()]
    }

    fn get_columns(&self) -> Vec<Self::Column> {
        self.columns.clone()
    }

    fn get_columns_names(&self) -> Vec<String> {
        self.columns.iter().map(|c| c.name.clone()).collect()
    }

    fn get_size(&self) -> i64 {
        self.size
    }

    fn get_column_indexes(&self) -> &HashMap<String, usize> {
        &self.column_indexes
    }

    fn get_dep_table_names(&self) -> Vec<String> {
        self.foreign_keys
            .iter()
            .map(|fk| format!("{}.{}", fk.foreign_table_schema, fk.foreign_table_name))
            .collect()
    }

    fn default_query(&self) -> String {
        self.query_with_select(vec![], None)
    }

    // explicit columns: generated ones are skipped
    fn query_with_select(&self, cs: Vec<Option<String>>, limit: Option<u64>) -> String {
        format!(
            "SELECT {} FROM {}{}{}",
            self.quoted_columns(),
            self.quoted_full_name(),
            sql_conditions(cs),
            sql_limit(limit),
        )
    }
}

impl MyTable {
    pub fn new(tablename: String, schemaname: String) -> Self {
        Self {
            tablename,
            schemaname,
            columns: vec![],
            foreign_keys: vec![],
            column_indexes: HashMap::new(),
            size: 0,
        }
    }

    /// Columns must be ordered by position (the data query selects them in this order)
    pub fn set_columns(&mut self, columns: Vec<MyColumn>) {
        self.column_indexes = columns
            .iter()
            .enumerate()
            .map(|(i, c)| (c.name.clone(), i))
            .collect();
        self.columns = columns;
    }

    pub fn quoted_full_name(&self) -> String {
        format!(
            "{}.{}",
            quote_ident(&self.schemaname),
            quote_ident(&self.tablename)
        )
    }

    pub fn quoted_columns(&self) -> String {
        self.columns
            .iter()
            .map(|c| quote_ident(&c.name))
            .collect::<Vec<_>>()
            .join(", ")
    }
}

pub fn quote_ident(name: &str) -> String {
    format!("`{}`", name.replace('`', "``"))
}
