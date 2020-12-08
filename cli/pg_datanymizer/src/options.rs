use anyhow::{anyhow, Result};
use structopt::StructOpt;
use url::Url;

#[derive(StructOpt, Debug, Clone)]
#[structopt(name = "pg_datanymizer")]
pub struct Options {
    #[structopt(name = "DBNAME")]
    database: String,

    #[structopt(
        short = "c",
        long = "config",
        help = "Path to config file. Default: ./config.yml"
    )]
    pub config: Option<String>,
    #[structopt(
        short = "f",
        long = "file",
        name = "FILE",
        help = "Path to dump file, example: /tmp/dump.sql"
    )]
    pub file: Option<String>,

    #[structopt(
        short = "d",
        long = "dbname",
        help = "database to dump",
        default_value = "postgres"
    )]
    pub db_name: String,
    #[structopt(
        short = "h",
        long = "host",
        help = "database server host or socket directory",
        default_value = "localhost"
    )]
    pub host: String,
    #[structopt(short = "p", long = "port", help = "database server port number")]
    pub port: Option<u16>,
    #[structopt(
        short = "U",
        long = "username",
        help = "connect as specified database user"
    )]
    pub username: Option<String>,
    #[structopt(
        short = "W",
        long = "password",
        help = "force password prompt (should happen automatically)"
    )]
    pub password: Option<String>,
    #[structopt(
        long = "pg_dump",
        help = "pg_dump file location",
        default_value = "pg_dump"
    )]
    pub pg_dump_location: String,
}

impl Options {
    pub fn database_url(&self) -> Result<String> {
        if let Ok(url) = Url::parse(self.database.as_str()) {
            if url.scheme() == "postgres" {
                return Ok(url.to_string());
            } else {
                return Err(anyhow!("Scheme url error"));
            }
        }
        self.build_url(Some(self.database.to_string()).filter(|x| !x.is_empty()))
    }

    fn build_url(&self, override_db_name: Option<String>) -> Result<String> {
        let db_name = override_db_name.unwrap_or_else(|| self.db_name.clone());
        if db_name.is_empty() {
            return Err(anyhow!("No one databae passed"));
        }

        let mut url = Url::parse(format!("postgres://{}", self.host).as_str())?;
        url.set_port(self.port)
            .map_err(|_| anyhow!("Cannot set port"))?;

        url.set_username(self.username.as_deref().unwrap_or_default())
            .map_err(|_| anyhow!("Cannot set username"))?;

        url.set_password(self.password.as_deref())
            .map_err(|_| anyhow!("Cannot set password"))?;

        url.set_path(&db_name);

        Ok(url.to_string())
    }
}

#[cfg(test)]
mod test {
    use super::Options;

    #[test]
    fn parse_empty_config() {
        let cfg = Options {
            database: "postgres://hostname/test".to_string(),
            config: None,
            file: None,
            db_name: "test".to_string(),
            host: "localhost".to_string(),
            port: None,
            username: None,
            password: None,
            pg_dump_location: "pg_dump".to_string(),
        };

        let expected = "postgres://hostname/test".to_string();
        assert_eq!(cfg.database_url().unwrap(), expected);
    }

    #[test]
    fn parse_empty_url() {
        let cfg1 = Options {
            database: String::default(),
            config: None,
            file: None,
            db_name: "test".to_string(),
            host: "hostname".to_string(),
            port: None,
            username: None,
            password: None,
            pg_dump_location: "pg_dump".to_string(),
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
        assert_eq!(cfg1.database_url().unwrap(), expected1);
        assert_eq!(cfg2.database_url().unwrap(), expected2);
        assert_eq!(cfg3.database_url().unwrap(), expected3);
        assert_eq!(cfg4.database_url().unwrap(), expected4);
    }
}
