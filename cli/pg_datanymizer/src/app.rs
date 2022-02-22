use anyhow::Result;
use std::time::Duration;
use std::{fs::File, io};
use update_informer::{registry::GitHub, Check};
use url::Url;

use crate::options::{Options, TransactionConfig};

use datanymizer_dumper::{
    indicator::{ConsoleIndicator, SilentIndicator},
    postgres::{connector::Connector, dumper::PgDumper, IsolationLevel},
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
        let mut connection = self.connector().connect()?;
        let engine = self.engine()?;

        if self.options.check_updates {
            self.print_new_version_if_available();
            return Ok(());
        }

        match &self.options.file {
            Some(filename) => PgDumper::new(
                engine,
                self.dump_isolation_level(),
                self.options.pg_dump_location.clone(),
                File::create(filename)?,
                ConsoleIndicator::new(),
                self.options.pg_dump_args.clone(),
            )?
            .dump(&mut connection),

            None => PgDumper::new(
                engine,
                self.dump_isolation_level(),
                self.options.pg_dump_location.clone(),
                io::stdout(),
                SilentIndicator,
                self.options.pg_dump_args.clone(),
            )?
            .dump(&mut connection),
        }
    }

    fn connector(&self) -> Connector {
        let options = &self.options;
        Connector::new(
            self.database_url.clone(),
            options.accept_invalid_hostnames,
            options.accept_invalid_certs,
        )
    }

    fn engine(&self) -> Result<Engine> {
        let settings = Settings::new(self.options.config.clone())?;
        Ok(Engine::new(settings))
    }

    fn dump_isolation_level(&self) -> Option<IsolationLevel> {
        match self.options.dump_transaction {
            TransactionConfig::NoTransaction => None,
            TransactionConfig::ReadUncommitted => Some(IsolationLevel::ReadUncommitted),
            TransactionConfig::ReadCommitted => Some(IsolationLevel::ReadCommitted),
            TransactionConfig::RepeatableRead => Some(IsolationLevel::RepeatableRead),
            TransactionConfig::Serializable => Some(IsolationLevel::Serializable),
        }
    }

    fn print_new_version_if_available(&self) {
        let pkg = "datanymizer";
        let repo = format!("{pkg}/{pkg}");
        let current_version = env!("CARGO_PKG_VERSION");

        let informer = update_informer::UpdateInformer::new(GitHub, &repo, current_version)
            .interval(Duration::ZERO);

        if let Ok(Some(version)) = informer.check_version() {
            let msg =
                format!("A new release of {pkg} is available: v{current_version} -> {version}");

            let release_url = format!("https://github.com/{repo}/releases/tag/{version}");

            println!("\n{msg}\n{release_url}");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use structopt::StructOpt;

    mod isolation_level {
        use super::*;

        #[test]
        fn default() {
            let options =
                Options::from_iter(vec!["DBNAME", "postgres://postgres@localhost/dbname"]);
            let level = App::from_options(options).unwrap().dump_isolation_level();
            assert!(matches!(level, Some(IsolationLevel::ReadCommitted)));
        }

        fn level(dt: &str) -> Option<IsolationLevel> {
            let options = Options::from_iter(vec![
                "DBNAME",
                "postgres://postgres@localhost/dbname",
                "--dump-transaction",
                dt,
            ]);
            App::from_options(options).unwrap().dump_isolation_level()
        }

        #[test]
        fn no_transaction() {
            let level = level("NoTransaction");
            assert!(level.is_none());
        }

        #[test]
        fn repeatable_read() {
            let level = level("RepeatableRead");
            assert!(matches!(level, Some(IsolationLevel::RepeatableRead)));
        }
    }
}
