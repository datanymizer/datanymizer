use anyhow::Result;
use datanymizer_dumper::postgres::dumper::PgDumper;
use datanymizer_dumper::Dumper;
use datanymizer_engine::{Engine, Settings};
use native_tls::TlsConnector;
use options::Options;
use postgres::Client;
use postgres_native_tls::MakeTlsConnector;
use structopt::StructOpt;

mod options;

fn main() -> Result<()> {
    let cfg = Options::from_args();

    let url = cfg.database_url()?;
    let s = Settings::new(
        cfg.clone()
            .config
            .unwrap_or_else(|| "./config.yml".to_string()),
        url.clone(),
    )?;

    let connector = TlsConnector::new()?;
    let connector = MakeTlsConnector::new(connector);
    let mut client = Client::connect(&url, connector)?;
    let mut dumper = PgDumper::new(Engine::new(s), cfg.pg_dump_location, cfg.file)?;
    dumper.dump(&mut client)
}
