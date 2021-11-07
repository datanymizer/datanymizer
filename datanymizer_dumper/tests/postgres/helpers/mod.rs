use postgres::{Client, NoTls};
use std::{
    env,
    fs::File,
    process::{Child, ChildStdin, Command, Stdio},
    sync::Once,
};
use url::Url;

static CREATE_SRC_DB: Once = Once::new();

const DB_ENV_KEY: &str = "DATANYMIZER_TEST_PG_DB";
const PG_DUMP_PATH_KEY: &str = "DATANYMIZER_TEST_PG_DUMP_PATH";
const PSQL_PATH_KEY: &str = "DATANYMIZER_TEST_PSQL_PATH";
const SRC_DUMP_PATH: &str = "tests/postgres/dumps/common.sql";

pub struct DstWrapper(Child);

impl DstWrapper {
    pub fn io(&mut self) -> ChildStdin {
        self.0.stdin.take().unwrap()
    }

    pub fn close(&mut self) {
        self.0.kill().unwrap();
    }
}

pub fn pg_dump_path() -> String {
    env::var(PG_DUMP_PATH_KEY).unwrap_or("pg_dump".to_string())
}

pub fn src_database_url() -> Url {
    Url::parse(
        env::var(DB_ENV_KEY)
            .expect(format!("No {} environment variable", DB_ENV_KEY).as_str())
            .as_str(),
    )
    .expect("Invalid database URL")
}

pub fn dst_database_url(name: &str) -> Url {
    let db_suffix = format!("fake_{}", name);
    let mut database_url = src_database_url();
    database_url.set_path(format!("{}_{}", database_url.path(), db_suffix).as_str());

    database_url
}

pub fn create_src_db() {
    CREATE_SRC_DB.call_once(|| {
        let file =
            File::open(SRC_DUMP_PATH).expect(format!("No dump file at {}", SRC_DUMP_PATH).as_str());
        let database_url = src_database_url();

        create_db(&database_url);

        psql_command()
            .arg(database_url.as_str())
            .stdin(file)
            .status()
            .expect("Error when restoring the test source database");
    });
}

pub fn src_connection() -> Client {
    create_src_db();
    connection(&src_database_url())
}

pub fn dst_connection(name: &str) -> Client {
    connection(&dst_database_url(name))
}

pub fn dst_wrapper(name: &str) -> DstWrapper {
    create_src_db();

    let dst_url = dst_database_url(name);
    create_db(&dst_url);

    DstWrapper(
        psql_command()
            .arg(dst_url.as_str())
            .stdin(Stdio::piped())
            .spawn()
            .unwrap(),
    )
}

fn create_db(url: &Url) {
    let db_name = url.path_segments().unwrap().next().unwrap().to_string();

    let mut new_database_url = url.clone();
    new_database_url.set_path("");

    run_sql(
        format!("DROP DATABASE IF EXISTS {};", db_name).as_str(),
        new_database_url.as_str(),
    );
    run_sql(
        format!("CREATE DATABASE {};", db_name).as_str(),
        new_database_url.as_str(),
    );
}

fn psql_command() -> Command {
    let psql_path = env::var(PSQL_PATH_KEY).unwrap_or("psql".to_string());
    let mut cmd = Command::new(psql_path);
    cmd.stdout(Stdio::null());
    cmd
}

fn run_sql(cmd: &str, db_url: &str) {
    psql_command()
        .args(&["-c", cmd])
        .arg(db_url)
        .status()
        .expect(
            format!(
                "Error when running sql command, db: {}, cmd: {}",
                db_url, cmd
            )
            .as_str(),
        );
}

fn connection(url: &Url) -> Client {
    Client::connect(url.as_str(), NoTls).unwrap()
}
