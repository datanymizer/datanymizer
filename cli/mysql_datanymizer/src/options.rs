use anyhow::{anyhow, Result};
use clap::{ArgAction, Parser};
use url::Url;

#[derive(Parser, Debug, Clone, Default)]
#[command(
    name = "mysql_datanymizer",
    about = "Powerful MySQL database anonymizer with flexible rules",
    version,
    disable_help_flag = true
)]
pub struct Options {
    #[arg(long, action = ArgAction::HelpLong)]
    help: Option<bool>,

    #[arg(name = "DBNAME", env = "MYSQLDATABASE")]
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
        long,
        help = "Database server host",
        default_value = "localhost",
        env = "MYSQLHOST"
    )]
    pub host: String,

    #[arg(
        short = 'P',
        long,
        help = "Database server port number [default: 3306]",
        env = "MYSQLPORT"
    )]
    pub port: Option<u16>,

    #[arg(
        short,
        long,
        help = "Connect as specified database user",
        env = "MYSQLUSER"
    )]
    pub user: Option<String>,

    #[arg(short, long, help = "User password", env = "MYSQLPASSWORD")]
    pub password: Option<String>,

    #[arg(
        long = "mysqldump",
        help = "mysqldump file location",
        default_value = "mysqldump"
    )]
    pub mysqldump_location: String,

    #[arg(
        long = "accept-invalid-hostnames",
        help = "Accept or not invalid hostnames when using SSL"
    )]
    pub accept_invalid_hostnames: bool,

    #[arg(
        long = "accept-invalid-certs",
        help = "Accept or not invalid certificates (e.g., self-signed) when using SSL"
    )]
    pub accept_invalid_certs: bool,

    #[arg(
        name = "MYSQLDUMP_ARGS",
        help = "The remaining arguments are passed directly to `mysqldump` calls. You should add `--` before <DBNAME> in such cases"
    )]
    pub mysqldump_args: Vec<String>,

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
                "mysql" => Ok(url),
                _ => Err(anyhow!("Scheme url error")),
            };
        }
        if self.database.is_empty() {
            Err(anyhow!("Database url is empty"))
        } else {
            self.build_url(self.database.to_string())
        }
    }

    fn build_url(&self, override_db_name: String) -> Result<Url> {
        let mut url = Url::parse(format!("mysql://{}", self.host).as_str())?;
        url.set_port(self.port)
            .map_err(|_| anyhow!("Cannot set port"))?;

        url.set_username(self.user.as_deref().unwrap_or_default())
            .map_err(|_| anyhow!("Cannot set username"))?;

        url.set_password(self.password.as_deref())
            .map_err(|_| anyhow!("Cannot set password"))?;

        url.set_path(&override_db_name);

        Ok(url)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_empty_config() {
        let cfg = Options {
            database: "mysql://hostname/test".to_string(),
            config: "./config.yml".to_string(),
            host: "localhost".to_string(),
            mysqldump_location: "mysqldump".to_string(),
            ..Default::default()
        };

        let expected = "mysql://hostname/test".to_string();
        assert_eq!(cfg.database_url().unwrap().to_string(), expected);
    }

    #[test]
    fn with_db_name() {
        let cfg1 = Options {
            config: "./config.yml".to_string(),
            database: "test".to_string(),
            host: "hostname".to_string(),
            mysqldump_location: "mysqldump".to_string(),
            ..Default::default()
        };

        let cfg2 = Options {
            port: Some(3307),
            ..cfg1.clone()
        };
        let cfg3 = Options {
            user: Some("test_user".to_string()),
            ..cfg2.clone()
        };
        let cfg4 = Options {
            password: Some("pass".to_string()),
            ..cfg3.clone()
        };

        let expected1 = "mysql://hostname/test".to_string();
        let expected2 = "mysql://hostname:3307/test".to_string();
        let expected3 = "mysql://test_user@hostname:3307/test".to_string();
        let expected4 = "mysql://test_user:pass@hostname:3307/test".to_string();
        assert_eq!(cfg1.database_url().unwrap().to_string(), expected1);
        assert_eq!(cfg2.database_url().unwrap().to_string(), expected2);
        assert_eq!(cfg3.database_url().unwrap().to_string(), expected3);
        assert_eq!(cfg4.database_url().unwrap().to_string(), expected4);
    }

    #[test]
    fn parse_args() {
        let cmd = vec![
            "mysql_datanymizer",
            "-c",
            "some_config.yml",
            "-f",
            "some_file.sql",
            "--",
            "mysql://user@hostname/test",
            "--insert-ignore",
            "--flush-logs",
        ];
        let options = Options::parse_from(cmd);

        assert_eq!(
            options.database_url().unwrap().as_str(),
            "mysql://user@hostname/test"
        );
        assert_eq!(options.config, "some_config.yml");
        assert_eq!(options.file, Some("some_file.sql".to_string()));
        assert_eq!(options.verbose, 0);
        assert!(!options.accept_invalid_hostnames);
        assert!(!options.accept_invalid_certs);
        assert_eq!(
            options.mysqldump_args,
            vec!["--insert-ignore", "--flush-logs"]
        );
    }

    #[test]
    fn verbose() {
        let cmd = vec!["mysql_datanymizer", "-v", "database"];
        let options = Options::parse_from(cmd);

        assert_eq!(options.verbose, 1);
    }

    #[test]
    fn very_verbose() {
        let cmd = vec!["mysql_datanymizer", "-vv", "database"];
        let options = Options::parse_from(cmd);

        assert_eq!(options.verbose, 2);
    }

    #[test]
    fn accept_invalid_hostnames() {
        let cmd = vec![
            "mysql_datanymizer",
            "--accept-invalid-hostnames",
            "database",
        ];
        let options = Options::parse_from(cmd);

        assert!(options.accept_invalid_hostnames);
    }
}
