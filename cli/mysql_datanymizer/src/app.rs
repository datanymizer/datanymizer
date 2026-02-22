use anyhow::Result;
use std::{
    fs::File,
    io::{self, Write},
};
use url::Url;

use crate::options::Options;

use datanymizer_dumper::{
    indicator::{ConsoleIndicator, Indicator, SilentIndicator},
    mysql::{connector::Connector, dumper::MysqlDumper},
    Dumper,
};
use datanymizer_engine::{Engine, Settings};

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

    pub fn run(&self) -> Result<()> {
        match (&self.options.file, &self.options.no_indicator) {
            (Some(filename), false) => {
                self.make_dump(File::create(filename)?, ConsoleIndicator::new())
            }
            (Some(filename), true) => self.make_dump(File::create(filename)?, SilentIndicator),
            _ => self.make_dump(io::stdout(), SilentIndicator),
        }
    }

    fn make_dump<W, I>(&self, w: W, i: I) -> Result<()>
    where
        W: 'static + Write + Send,
        I: 'static + Indicator + Send,
    {
        let mut connection = Connector::new(self.database_url.clone()).connect()?;
        let engine = self.engine()?;

        MysqlDumper::new(
            engine,
            self.options.mysqldump_location.clone(),
            w,
            i,
            self.options.mysqldump_args.clone(),
        )?
        .dump(&mut connection)
    }

    fn engine(&self) -> Result<Engine> {
        let settings = Settings::new(self.options.config.clone())?;
        Ok(Engine::new(settings))
    }
}
