use anyhow::Result;
use async_std::net::TcpStream;
use tiberius::{Config, SqlBrowser};
use url::Url;

type Client = tiberius::Client<TcpStream>;

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
}

impl Connector {
    pub fn new(url: Url) -> Self {
        Self { url }
    }

    pub async fn connect(&self) -> Result<Connection> {
        let url_str = self.url.as_str();

        let config = Config::from_ado_string(url_str)?;
        let tcp = TcpStream::connect_named(&config).await?;
        let client = Client::connect(config, tcp).await?;

        Ok(Connection::new(client, self.url.clone()))
    }
}
