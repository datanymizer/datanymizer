use anyhow::Result;
use datanymizer_dumper::{
    mssql::dumper::{MsSqlClient, MsSqlDumper},
    Dumper,
};
use datanymizer_engine::{Engine, Settings};
use structopt::StructOpt;

mod options;
use options::Options;

fn main() -> Result<()> {
    let cfg = Options::from_args();

    let url = cfg.database_url()?;
    let s = Settings::new(
        cfg.clone()
            .config
            .unwrap_or_else(|| "./config.yml".to_string()),
        url.clone(),
    )?;

    let mut client = MsSqlClient;
    let mut dumper = MsSqlDumper::new(Engine::new(s), cfg.mssql_scripter_location, cfg.file)?;
    dumper.dump(&mut client)
}
