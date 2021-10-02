use anyhow::Result;
use native_tls::TlsConnector;
use postgres::{Client, IsolationLevel, NoTls};
use postgres_native_tls::MakeTlsConnector;
use std::borrow::Cow;
use url::Url;

use crate::options::{Options, TransactionConfig};

use datanymizer_dumper::{postgres::dumper::PgDumper, Dumper};
use datanymizer_engine::{Engine, Settings};

const SSL_MODE_PARAM: &str = "sslmode";
const NO_SSL_MODE: &str = "disable";

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
        let mut client = self.client()?;
        let mut dumper = self.dumper()?;

        dumper.dump(&mut client)
    }

    fn dumper(&self) -> Result<PgDumper> {
        let engine = self.engine()?;

        PgDumper::new(
            engine,
            self.dump_isolation_level(),
            self.options.pg_dump_location.clone(),
            self.options.file.clone(),
            self.options.pg_dump_args.clone(),
        )
    }

    fn engine(&self) -> Result<Engine> {
        let settings = Settings::new(self.options.config.clone(), self.database_url.to_string())?;
        Ok(Engine::new(settings))
    }

    fn client(&self) -> Result<Client> {
        let url = self.database_url.to_string();
        let client = match self.tls_connector()? {
            Some(c) => Client::connect(&url, c)?,
            None => Client::connect(&url, NoTls)?,
        };

        Ok(client)
    }

    fn tls_connector(&self) -> Result<Option<MakeTlsConnector>> {
        let ssl_mode = self
            .database_url
            .query_pairs()
            .find_map(|(key, value)| {
                if key == SSL_MODE_PARAM {
                    Some(value)
                } else {
                    None
                }
            })
            .unwrap_or(Cow::Borrowed(NO_SSL_MODE));

        let connector = if ssl_mode == NO_SSL_MODE {
            None
        } else {
            let mut builder = TlsConnector::builder();

            if self.options.accept_invalid_hostnames {
                builder.danger_accept_invalid_hostnames(true);
            }
            if self.options.accept_invalid_certs {
                builder.danger_accept_invalid_certs(true);
            }

            let connector = builder.build()?;
            Some(MakeTlsConnector::new(connector))
        };

        Ok(connector)
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

    mod tls_connector {
        use super::*;

        fn connector(db_str: &str) -> Option<MakeTlsConnector> {
            let options = Options::from_iter(vec!["DBNAME", db_str]);
            let app = App::from_options(options).unwrap();
            app.tls_connector().unwrap()
        }

        #[test]
        fn default() {
            let connector = connector("postgres://postgres@localhost/dbname");
            assert!(connector.is_none());
        }

        #[test]
        fn ssl_disable() {
            let connector = connector("postgres://postgres@localhost/dbname?sslmode=disable");
            assert!(connector.is_none());
        }

        #[test]
        fn ssl_prefer() {
            let connector = connector("postgres://postgres@localhost/dbname?sslmode=prefer");
            assert!(connector.is_some());
        }

        #[test]
        fn ssl_require() {
            let connector = connector("postgres://postgres@localhost/dbname?sslmode=require");
            assert!(connector.is_some());
        }
    }

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
