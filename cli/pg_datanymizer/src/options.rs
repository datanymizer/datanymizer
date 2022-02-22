use anyhow::{anyhow, Result};
use structopt::{clap::arg_enum, StructOpt};
use url::Url;

arg_enum! {
    #[derive(Debug, Clone)]
    pub enum TransactionConfig {
        NoTransaction,
        ReadUncommitted,
        ReadCommitted,
        RepeatableRead,
        Serializable,
    }
}

impl Default for TransactionConfig {
    fn default() -> Self {
        Self::ReadCommitted
    }
}

#[derive(StructOpt, Debug, Clone, Default)]
#[structopt(name = "pg_datanymizer")]
pub struct Options {
    #[structopt(name = "DBNAME")]
    database: String,

    #[structopt(
        short,
        long,
        help = "Path to config file",
        default_value = "./config.yml"
    )]
    pub config: String,

    #[structopt(
        short,
        long,
        name = "FILE",
        help = "Path to dump file, example: /tmp/dump.sql"
    )]
    pub file: Option<String>,

    #[structopt(
        short,
        long = "dbname",
        help = "database to dump",
        default_value = "postgres"
    )]
    pub db_name: String,

    #[structopt(
        short,
        long,
        help = "Database server host or socket directory",
        default_value = "localhost"
    )]
    pub host: String,

    #[structopt(short, long, help = "Database server port number [default: 5432]")]
    pub port: Option<u16>,

    #[structopt(short = "U", long, help = "Connect as specified database user")]
    pub username: Option<String>,

    #[structopt(short = "W", long, help = "User password")]
    pub password: Option<String>,

    #[structopt(
        long,
        default_value,
        case_insensitive = true,
        possible_values = &TransactionConfig::variants(),
        help = "Using a transaction when dumping data, you can specify the isolation level",
    )]
    pub dump_transaction: TransactionConfig,

    #[structopt(
        long = "pg_dump",
        help = "pg_dump file location",
        default_value = "pg_dump"
    )]
    pub pg_dump_location: String,

    #[structopt(
        long = "accept_invalid_hostnames",
        help = "Accept or not invalid hostnames when using SSL"
    )]
    pub accept_invalid_hostnames: bool,

    #[structopt(
        long = "accept_invalid_certs",
        help = "Accept or not invalid certificates (e.g., self-signed) when using SSL"
    )]
    pub accept_invalid_certs: bool,

    #[structopt(
        name = "PG_DUMP_ARGS",
        help = "The remaining arguments are passed directly to `pg_dump` calls. You should add `--` before <DBNAME> in such cases"
    )]
    pub pg_dump_args: Vec<String>,

    #[structopt(long = "check_updates", help = "Check for updates")]
    pub check_updates: bool,
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
        let options = Options::from_iter(cmd);

        assert_eq!(
            options.database_url().unwrap().as_str(),
            "postgres://user@hostname/test"
        );
        assert_eq!(options.config, "some_config.yml");
        assert_eq!(options.file, Some("some_file.sql".to_string()));
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
}
