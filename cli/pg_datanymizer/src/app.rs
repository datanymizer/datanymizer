use anyhow::Result;
use std::{
    fs::File,
    io::{self, Write},
};
use url::Url;

use crate::options::{Options, TransactionConfig};

use datanymizer_dumper::{
    indicator::{ConsoleIndicator, Indicator, SilentIndicator},
    mysql::{self, dumper::IsolationLevel as MyIsolationLevel},
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
        if self.database_url.scheme() == "mysql" {
            return self.make_mysql_dump(w, i);
        }
        let mut connection = self.connector().connect()?;
        let engine = self.engine()?;
        let dump_isolation_level = self.dump_isolation_level();

        PgDumper::new(
            engine,
            dump_isolation_level,
            self.options.pg_dump_location.clone(),
            w,
            i,
            self.options.pg_dump_args.clone(),
        )?
        .dump(&mut connection)
    }

    fn make_mysql_dump<W, I>(&self, w: W, i: I) -> Result<()>
    where
        W: 'static + Write + Send,
        I: 'static + Indicator + Send,
    {
        let options = &self.options;
        let mut connection = mysql::Connector::new(
            self.database_url.clone(),
            options.accept_invalid_hostnames,
            options.accept_invalid_certs,
        )
        .connect()?;
        mysql::dumper::MyDumper::new(
            self.engine()?,
            self.mysql_isolation_level(),
            options.mysqldump_location.clone(),
            w,
            i,
            options.pg_dump_args.clone(),
        )?
        .dump(&mut connection)
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

    fn mysql_isolation_level(&self) -> Option<MyIsolationLevel> {
        match self.options.dump_transaction {
            TransactionConfig::NoTransaction => None,
            TransactionConfig::ReadUncommitted => Some(MyIsolationLevel::ReadUncommitted),
            TransactionConfig::ReadCommitted => Some(MyIsolationLevel::ReadCommitted),
            TransactionConfig::RepeatableRead => Some(MyIsolationLevel::RepeatableRead),
            TransactionConfig::Serializable => Some(MyIsolationLevel::Serializable),
        }
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

    mod isolation_level {
        use super::*;
        use clap::Parser;

        #[test]
        fn default() {
            let options =
                Options::parse_from(vec!["DBNAME", "postgres://postgres@localhost/dbname"]);
            let level = App::from_options(options).unwrap().dump_isolation_level();
            assert!(matches!(level, Some(IsolationLevel::ReadCommitted)));
        }

        fn level(dt: &str) -> Option<IsolationLevel> {
            let options = Options::parse_from(vec![
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

    mod mysql_isolation_level {
        use super::*;
        use clap::Parser;

        fn level(dt: &str) -> Option<MyIsolationLevel> {
            let options = Options::parse_from(vec![
                "DBNAME",
                "mysql://root@localhost/dbname",
                "--dump-transaction",
                dt,
            ]);
            App::from_options(options).unwrap().mysql_isolation_level()
        }

        #[test]
        fn all() {
            assert!(level("NoTransaction").is_none());
            assert!(matches!(
                level("ReadUncommitted"),
                Some(MyIsolationLevel::ReadUncommitted)
            ));
            assert!(matches!(
                level("ReadCommitted"),
                Some(MyIsolationLevel::ReadCommitted)
            ));
            assert!(matches!(
                level("RepeatableRead"),
                Some(MyIsolationLevel::RepeatableRead)
            ));
            assert!(matches!(
                level("Serializable"),
                Some(MyIsolationLevel::Serializable)
            ));
        }
    }

    /// Runs only with the `DATANYMIZER_TEST_MYSQL_DB` env variable (see docs/integration_tests.md)
    mod mysql_dump {
        use super::*;
        use ::mysql::{prelude::Queryable, Conn, Opts};
        use clap::Parser;
        use std::{env, fs};

        #[test]
        fn dump() {
            let Ok(db) = env::var("DATANYMIZER_TEST_MYSQL_DB") else {
                return;
            };
            let mut url = Url::parse(&db).unwrap();
            url.set_path(&format!("{}_cli", url.path()));
            let name = url.path().trim_start_matches('/').to_string();

            let mut server_url = url.clone();
            server_url.set_path("");
            let mut conn = Conn::new(Opts::from_url(server_url.as_str()).unwrap()).unwrap();
            conn.query_drop(format!("DROP DATABASE IF EXISTS `{}`", name))
                .unwrap();
            conn.query_drop(format!("CREATE DATABASE `{}`", name))
                .unwrap();
            conn.query_drop(format!(
                "CREATE TABLE `{}`.users (id INT PRIMARY KEY, email VARCHAR(100))",
                name
            ))
            .unwrap();
            conn.query_drop(format!(
                "INSERT INTO `{}`.users VALUES (1, 'secret@corp.com')",
                name
            ))
            .unwrap();

            let dir = env::temp_dir().join(format!("datanymizer_cli_{}", std::process::id()));
            fs::create_dir_all(&dir).unwrap();
            let config = dir.join("config.yml");
            fs::write(
                &config,
                "tables:\n  - name: users\n    rules:\n      email:\n        email: {}\n",
            )
            .unwrap();
            let file = dir.join("dump.sql");

            let options = Options::parse_from(vec![
                "pg_datanymizer".to_string(),
                "-c".into(),
                config.to_string_lossy().into(),
                "-f".into(),
                file.to_string_lossy().into(),
                "--no-indicator".into(),
                "--mysqldump".into(),
                env::var("DATANYMIZER_TEST_MYSQLDUMP_PATH").unwrap_or("mysqldump".into()),
                url.to_string(),
            ]);
            App::from_options(options).unwrap().run().unwrap();

            let dump = fs::read_to_string(&file).unwrap();
            fs::remove_dir_all(&dir).unwrap();
            assert!(dump.contains("CREATE TABLE `users`"));
            assert!(dump.contains("INSERT INTO `users` (`id`, `email`) VALUES"));
            assert!(!dump.contains("secret@corp.com"));
        }
    }
}
