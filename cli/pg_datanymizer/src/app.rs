use anyhow::Result;
use url::Url;

use crate::options::{Options, TransactionConfig};

use datanymizer_dumper::{
    postgres::{connector::Connector, dumper::PgDumper, writer::DumpWriter, IsolationLevel},
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
        let mut dumper = self.dumper()?;
        let mut client = self.connector().connect()?;

        dumper.dump(&mut client)
    }

    fn connector(&self) -> Connector {
        let options = &self.options;
        Connector::new(
            self.database_url.clone(),
            options.accept_invalid_hostnames,
            options.accept_invalid_certs,
        )
    }

    fn dumper(&self) -> Result<PgDumper> {
        let engine = self.engine()?;

        PgDumper::new(
            engine,
            self.dump_isolation_level(),
            self.options.pg_dump_location.clone(),
            DumpWriter::new(self.options.file.clone())?,
            self.options.pg_dump_args.clone(),
        )
    }

    fn engine(&self) -> Result<Engine> {
        let settings = Settings::new(self.options.config.clone(), self.database_url.to_string())?;
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
