use anyhow::Result;
use async_std::net::TcpStream;
use datanymizer_dumper::{mssql::dumper::MsSqlDumper, Dumper};
use datanymizer_engine::{Engine, Settings};
use structopt::StructOpt;
use tiberius::{Client, Config, SqlBrowser};

mod options;
use options::Options;

#[async_std::main]
async fn main() -> Result<()> {
    let cfg = Options::from_args();

    let connection_string = cfg.connection_string()?;
    let s = Settings::new(
        cfg.clone()
            .config
            .unwrap_or_else(|| "./config.yml".to_string()),
        connection_string.clone(),
    )?;

    let config = Config::from_ado_string(&connection_string)?;
    let tcp = TcpStream::connect_named(&config).await?;
    let mut client = Client::connect(config, tcp).await?;

    let mut dumper = MsSqlDumper::new(Engine::new(s), cfg.mssql_scripter_location, cfg.file)?;
    dumper.dump(&mut client)
}
