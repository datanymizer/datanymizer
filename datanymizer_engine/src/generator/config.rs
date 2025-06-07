use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub tables: Vec<Table>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Table {
    pub name: String,
    pub row_count: usize,
    pub key: Option<PrimaryKey>,
    pub foreign_keys: Vec<ForeignKey>,
}

#[derive(Debug, Deserialize, Clone)]
pub enum Key {
    Primary(PrimaryKey),
    Foreign(ForeignKey),
}

#[derive(Debug, Deserialize, Clone)]
pub struct PrimaryKey {
    pub name: String,
    pub table_name: Option<String>,

    #[serde(flatten)]
    pub options: HashMap<String, String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ForeignKey {
    pub name: String,
    pub table_name: String,
    pub kind: ForeignKeyKind,
    pub source: Box<Key>,

    #[serde(flatten)]
    pub options: HashMap<String, String>,
}

#[derive(Debug, Deserialize, Clone, Default)]
#[serde(rename_all = "snake_case")]
pub enum ForeignKeyKind {
    Monotonic,
    Random,
    #[default]
    MonotonicRandom,
}
