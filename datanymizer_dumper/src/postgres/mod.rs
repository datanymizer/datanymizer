use crate::{Dumper, SchemaInspector};

pub mod column;
pub mod dumper;
mod escaper;
pub mod foreign_key;
mod query_wrapper;
pub mod row;
pub mod schema_inspector;
mod sequence;
pub mod table;
pub mod writer;

pub struct PostgresDatabase;
