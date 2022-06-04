use anyhow::Result;
use std::{fs::File, io};
use url::Url;

use crate::options::Options;

use datanymizer_dumper::{
    indicator::{ConsoleIndicator, SilentIndicator},
    Dumper,
};
use datanymizer_engine::{Engine, Settings};
use mssql_datanymizer_dumper::{connector::Connector, dumper::MsSqlDumper};

pub struct App {
    options: Options,
    database_url: Url,
}

impl App {
    pub fn from_options(options: Options) -> Result<Self> {
        let database_url = options.database_url()?;

        Ok(App {
            options,
            database_url,
        })
    }

    pub async fn run(&self) -> Result<()> {
        let mut connection = self.connector().connect().await?;
        let engine = self.engine()?;

        match &self.options.file {
            Some(filename) => MsSqlDumper::new(
                engine,
                self.options.mssql_scripter_location.clone(),
                File::create(filename)?,
                ConsoleIndicator::new(),
            )?
            .dump(&mut connection),

            None => MsSqlDumper::new(
                engine,
                self.options.mssql_scripter_location.clone(),
                io::stdout(),
                SilentIndicator,
            )?
            .dump(&mut connection),
        }
    }

    fn connector(&self) -> Connector {
        Connector::new(self.database_url.clone())
    }

    fn engine(&self) -> Result<Engine> {
        let settings = Settings::new(self.options.config.clone())?;
        Ok(Engine::new(settings))
    }
}
