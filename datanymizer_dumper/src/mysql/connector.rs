use sqlx::{Connection as _, MySqlConnection};
use tokio::runtime::{Builder, Runtime};
use url::Url;

pub struct Connection {
    pub url: Url,
    pub conn: MySqlConnection,
    pub rt: Runtime,
}

impl Connection {
    pub fn new(url: Url, conn: MySqlConnection, rt: Runtime) -> Self {
        Self { url, conn, rt }
    }

    pub fn db_name(&self) -> &str {
        self.url
            .path_segments()
            .expect("Url should contain DB name")
            .collect::<Vec<_>>()[0]
    }
}

pub struct Connector {
    url: Url,
}

impl Connector {
    pub fn new(url: Url) -> Self {
        Self { url }
    }

    pub fn connect(&self) -> anyhow::Result<Connection> {
        let rt = Builder::new_current_thread().enable_all().build()?;
        let conn = rt.block_on(MySqlConnection::connect(self.url.as_str()))?;
        Ok(Connection::new(self.url.clone(), conn, rt))
    }
}
