use anyhow::Result;
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
        long = "mssql_scripter",
        help = "mssql_scripter location",
        default_value = "mssql_scripter"
    )]
    pub mssql_scripter_location: String,
}

impl Options {
    pub fn connection_string(&self) -> Result<String> {
        Ok(self.database.clone())
    }
}

#[cfg(test)]
mod test {}
