use super::helpers;

use pg_datanymizer_dumper::connector::Connector;

fn test_connection(tls_mode: &str) {
    let mut database_url = helpers::src_database_url();
    database_url.set_query(Some(format!("sslmode={}", tls_mode).as_str()));

    let connector = Connector::new(database_url.clone(), true, true);
    let mut connection = connector.connect().unwrap();
    let count: i64 = connection
        .client
        .query_one("SELECT COUNT(*) FROM actor", &[])
        .unwrap()
        .get(0);
    assert!(count > 0);
    assert_eq!(connection.url, database_url);
}

#[test]
fn connect() {
    helpers::create_src_db();

    test_connection("disable");
    test_connection("prefer");
    // requires TLS support at the test server, it is not yet implemented in the `ci.yml`
    // test_connection("require");
}
