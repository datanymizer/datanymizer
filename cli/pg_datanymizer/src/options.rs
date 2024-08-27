use anyhow::{anyhow, Result};
use clap::{ArgAction, Parser, ValueEnum};
use url::Url;

#[derive(ValueEnum, Debug, Clone)]
#[value(rename_all = "PascalCase")]
pub enum TransactionConfig {
    NoTransaction,
    ReadUncommitted,
    ReadCommitted,
    RepeatableRead,
    Serializable,
}

impl Default for TransactionConfig {
    fn default() -> Self {
        Self::ReadCommitted
    }
}

#[derive(Parser, Debug, Clone, Default)]
#[command(
    name = "pg_datanymizer",
    about = "Powerful Postgres database anonymizer with flexible rules",
    version,
    disable_help_flag = true
)]
pub struct Options {
    #[arg(long, action = ArgAction::HelpLong)]
    help: Option<bool>,

    #[arg(name = "DBNAME", env = "PGDATABASE")]
    database: String,

    #[arg(
        short,
        long,
        help = "Path to config file",
        default_value = "./config.yml"
    )]
    pub config: String,

    #[arg(
        short,
        long,
        name = "FILE",
        help = "Path to dump file, example: /tmp/dump.sql"
    )]
    pub file: Option<String>,

    #[arg(
        short,
        long = "dbname",
        help = "database to dump",
        default_value = "postgres"
    )]
    pub db_name: String,

    #[arg(
        short,
        long,
        help = "Database server host or socket directory",
        default_value = "localhost",
        env = "PGHOST"
    )]
    pub host: String,

    #[arg(
        short,
        long,
        help = "Database server port number [default: 5432]",
        env = "PGPORT"
    )]
    pub port: Option<u16>,

    #[arg(
        short = 'U',
        long,
        help = "Connect as specified database user",
        env = "PGUSER"
    )]
    pub username: Option<String>,

    #[arg(short = 'W', long, help = "User password", env = "PGPASSWORD")]
    pub password: Option<String>,

    #[arg(
        value_enum,
        long,
        default_value_t,
        ignore_case = true,
        help = "Using a transaction when dumping data, you can specify the isolation level"
    )]
    pub dump_transaction: TransactionConfig,

    #[arg(
        long = "pg_dump",
        help = "pg_dump file location",
        default_value = "pg_dump"
    )]
    pub pg_dump_location: String,

    #[arg(
        long = "accept_invalid_hostnames",
        help = "Accept or not invalid hostnames when using SSL"
    )]
    pub accept_invalid_hostnames: bool,

    #[arg(
        long = "accept_invalid_certs",
        help = "Accept or not invalid certificates (e.g., self-signed) when using SSL"
    )]
    pub accept_invalid_certs: bool,

    #[arg(
        name = "PG_DUMP_ARGS",
        help = "The remaining arguments are passed directly to `pg_dump` calls. You should add `--` before <DBNAME> in such cases"
    )]
    pub pg_dump_args: Vec<String>,

    #[arg(
        action = ArgAction::Count,
        short = 'v',
        help = "Turn on verbose logging features to get more information about dumper errors"
    )]
    pub verbose: u8,

    #[arg(long, name = "no-indicator", help = "Disable indicator")]
    pub no_indicator: bool,
}

impl Options {
    pub fn database_url(&self) -> Result<Url> {
        if let Ok(url) = Url::parse(self.database.as_str()) {
            return match url.scheme() {
                "postgres" | "postgresql" => Ok(url),
                _ => Err(anyhow!("Scheme url error")),
            };
        }
        self.build_url(Some(self.database.to_string()).filter(|x| !x.is_empty()))
    }

    fn build_url(&self, override_db_name: Option<String>) -> Result<Url> {
        let db_name = override_db_name.unwrap_or_else(|| self.db_name.clone());
        if db_name.is_empty() {
            return Err(anyhow!("No one database passed"));
        }

        let mut url = Url::parse(format!("postgres://{}", self.host).as_str())?;
        url.set_port(self.port)
            .map_err(|_| anyhow!("Cannot set port"))?;

        url.set_username(self.username.as_deref().unwrap_or_default())
            .map_err(|_| anyhow!("Cannot set username"))?;

        url.set_password(self.password.as_deref())
            .map_err(|_| anyhow!("Cannot set password"))?;

        url.set_path(&db_name);

        Ok(url)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_empty_config() {
        let cfg = Options {
            database: "postgres://hostname/test".to_string(),
            config: "./config.yml".to_string(),
            db_name: "test".to_string(),
            host: "localhost".to_string(),
            pg_dump_location: "pg_dump".to_string(),
            ..Default::default()
        };

        let expected = "postgres://hostname/test".to_string();
        assert_eq!(cfg.database_url().unwrap().to_string(), expected);
    }

    #[test]
    fn parse_empty_url() {
        let cfg1 = Options {
            config: "./config.yml".to_string(),
            db_name: "test".to_string(),
            host: "hostname".to_string(),
            pg_dump_location: "pg_dump".to_string(),
            ..Default::default()
        };

        let cfg2 = Options {
            port: Some(5433),
            ..cfg1.clone()
        };
        let cfg3 = Options {
            username: Some("test_user".to_string()),
            ..cfg2.clone()
        };
        let cfg4 = Options {
            password: Some("pass".to_string()),
            ..cfg3.clone()
        };

        let expected1 = "postgres://hostname/test".to_string();
        let expected2 = "postgres://hostname:5433/test".to_string();
        let expected3 = "postgres://test_user@hostname:5433/test".to_string();
        let expected4 = "postgres://test_user:pass@hostname:5433/test".to_string();
        assert_eq!(cfg1.database_url().unwrap().to_string(), expected1);
        assert_eq!(cfg2.database_url().unwrap().to_string(), expected2);
        assert_eq!(cfg3.database_url().unwrap().to_string(), expected3);
        assert_eq!(cfg4.database_url().unwrap().to_string(), expected4);
    }

    #[test]
    fn parse_args() {
        let cmd = vec![
            "pg_datanymizer",
            "-c",
            "some_config.yml",
            "-f",
            "some_file.sql",
            "--",
            "postgres://user@hostname/test",
            "--no-owner",
            "--no-acl",
        ];
        let options = Options::parse_from(cmd);

        assert_eq!(
            options.database_url().unwrap().as_str(),
            "postgres://user@hostname/test"
        );
        assert_eq!(options.config, "some_config.yml");
        assert_eq!(options.file, Some("some_file.sql".to_string()));
        assert_eq!(options.verbose, 0);
        assert!(!options.accept_invalid_hostnames);
        assert!(!options.accept_invalid_certs);
        assert!(matches!(
            options.dump_transaction,
            TransactionConfig::ReadCommitted
        ));
        assert_eq!(options.pg_dump_args, vec!["--no-owner", "--no-acl"]);
    }

    #[test]
    fn support_multiple_schemes() {
        let scheme1 = "postgres://user@hostname/test";
        let scheme2 = "postgresql://user@hostname/test";

        let opts1 = Options {
            database: scheme1.to_string(),
            ..Default::default()
        };
        let opts2 = Options {
            database: scheme2.to_string(),
            ..Default::default()
        };

        assert_eq!(opts1.database_url().unwrap().to_string(), scheme1);
        assert_eq!(opts2.database_url().unwrap().to_string(), scheme2);
    }

    #[test]
    fn dump_transaction_pascal_case() {
        let cmd = vec![
            "pg_datanymizer",
            "--dump-transaction",
            "RepeatableRead",
            "database",
        ];
        let options = Options::parse_from(cmd);

        assert!(matches!(
            options.dump_transaction,
            TransactionConfig::RepeatableRead
        ));
    }

    #[test]
    fn dump_transaction_lower_case() {
        let cmd = vec![
            "pg_datanymizer",
            "--dump-transaction",
            "repeatableread",
            "database",
        ];
        let options = Options::parse_from(cmd);

        assert!(matches!(
            options.dump_transaction,
            TransactionConfig::RepeatableRead
        ));
    }

    #[test]
    fn verbose() {
        let cmd = vec!["pg_datanymizer", "-v", "database"];
        let options = Options::parse_from(cmd);

        assert_eq!(options.verbose, 1);
    }

    #[test]
    fn very_verbose() {
        let cmd = vec!["pg_datanymizer", "-vv", "database"];
        let options = Options::parse_from(cmd);

        assert_eq!(options.verbose, 2);
    }

    #[test]
    fn accept_invalid_hostnames() {
        let cmd = vec!["pg_datanymizer", "--accept_invalid_hostnames", "database"];
        let options = Options::parse_from(cmd);

        assert!(options.accept_invalid_hostnames);
    }
}
