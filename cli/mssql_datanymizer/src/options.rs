use anyhow::{anyhow, Result};
use structopt::StructOpt;

#[derive(StructOpt, Debug, Clone)]
#[structopt(name = "mssql_datanymizer")]
pub struct Options {
    #[structopt(name = "DBNAME")]
    database: String,

    #[structopt(
        short = "c",
        long = "config",
        help = "Path to config file. Default: ./config.yml"
    )]
    pub config: Option<String>,

    #[structopt(
        short = "f",
        long = "file",
        name = "FILE",
        help = "Path to dump file, example: /tmp/dump.sql"
    )]
    pub file: Option<String>,

    #[structopt(
        short = "d",
        long = "dbname",
        help = "database to dump",
        default_value = "master"
    )]
    pub db_name: String,

    #[structopt(
        short = "h",
        long = "host",
        help = "database server host",
        default_value = "localhost"
    )]
    pub host: String,

    #[structopt(short = "p", long = "port", help = "database server port")]
    pub port: Option<u16>,

    #[structopt(
        short = "U",
        long = "username",
        help = "connect as specified database user"
    )]
    pub username: Option<String>,

    #[structopt(short = "W", long = "password", help = "Password")]
    pub password: Option<String>,

    #[structopt(
        long = "mssql_scripter",
        help = "mssql_scripter location",
        default_value = "mssql_scripter"
    )]
    pub mssql_scripter_location: String,
}

impl Options {
    pub fn database_url(&self) -> Result<String> {
        Ok(self.database.clone())
    }
}

#[cfg(test)]
mod test {}
