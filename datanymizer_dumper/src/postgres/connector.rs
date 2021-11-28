use anyhow::Result;
use native_tls::TlsConnector;
use postgres::{Client, NoTls};
use postgres_native_tls::MakeTlsConnector;
use std::borrow::Cow;
use url::Url;

const SSL_MODE_PARAM: &str = "sslmode";
const NO_SSL_MODE: &str = "disable";

pub struct Connection {
    pub client: Client,
    pub url: Url,
}

impl Connection {
    pub fn new(client: Client, url: Url) -> Self {
        Self { client, url }
    }
}

pub struct Connector {
    url: Url,
    accept_invalid_hostnames: bool,
    accept_invalid_certs: bool,
}

impl Connector {
    pub fn new(url: Url, accept_invalid_hostnames: bool, accept_invalid_certs: bool) -> Self {
        Self {
            url,
            accept_invalid_hostnames,
            accept_invalid_certs,
        }
    }

    pub fn connect(&self) -> Result<Connection> {
        let url_str = self.url.as_str();
        let client = match self.tls_connector()? {
            Some(c) => Client::connect(url_str, c)?,
            None => Client::connect(url_str, NoTls)?,
        };

        Ok(Connection::new(client, self.url.clone()))
    }

    fn tls_connector(&self) -> Result<Option<MakeTlsConnector>> {
        let ssl_mode = self
            .url
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

            if self.accept_invalid_hostnames {
                builder.danger_accept_invalid_hostnames(true);
            }
            if self.accept_invalid_certs {
                builder.danger_accept_invalid_certs(true);
            }

            let connector = builder.build()?;
            Some(MakeTlsConnector::new(connector))
        };

        Ok(connector)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod tls_connector {
        use super::*;

        fn tls_connector(db_str: &str) -> Option<MakeTlsConnector> {
            let c = Connector::new(Url::parse(db_str).unwrap(), false, false);
            c.tls_connector().unwrap()
        }

        #[test]
        fn default() {
            let tls_connector = tls_connector("postgres://postgres@localhost/dbname");
            assert!(tls_connector.is_none());
        }

        #[test]
        fn ssl_disable() {
            let tls_connector =
                tls_connector("postgres://postgres@localhost/dbname?sslmode=disable");
            assert!(tls_connector.is_none());
        }

        #[test]
        fn ssl_prefer() {
            let tls_connector =
                tls_connector("postgres://postgres@localhost/dbname?sslmode=prefer");
            assert!(tls_connector.is_some());
        }

        #[test]
        fn ssl_require() {
            let tls_connector =
                tls_connector("postgres://postgres@localhost/dbname?sslmode=require");
            assert!(tls_connector.is_some());
        }
    }
}
