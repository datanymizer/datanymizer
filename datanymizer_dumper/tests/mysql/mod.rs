#![cfg(feature = "mysql_db_tests")]

use datanymizer_dumper::{
    indicator::SilentIndicator,
    mysql::{
        dumper::{IsolationLevel, MyDumper},
        schema_inspector::MySchemaInspector,
        Connection, Connector,
    },
    Dumper, SchemaInspector, Table,
};
use datanymizer_engine::{Engine, Settings};
use mysql::{prelude::Queryable, Conn, Opts, Row, Value};
use std::{
    env,
    process::{Command, Stdio},
    sync::Once,
};
use url::Url;

const DB_ENV_KEY: &str = "DATANYMIZER_TEST_MYSQL_DB";
const MYSQLDUMP_PATH_KEY: &str = "DATANYMIZER_TEST_MYSQLDUMP_PATH";
const MYSQL_PATH_KEY: &str = "DATANYMIZER_TEST_MYSQL_PATH";

static CREATE_SRC_DB: Once = Once::new();

const SRC_SCHEMA: &[&str] = &[
    "CREATE TABLE users (
        id INT AUTO_INCREMENT PRIMARY KEY,
        login VARCHAR(100) NOT NULL UNIQUE,
        email VARCHAR(200),
        api_secret VARBINARY(32),
        bio TEXT,
        meta JSON,
        balance DECIMAL(10,2),
        flags BIT(3),
        created_at TIMESTAMP NULL,
        born DATETIME(3),
        login_upper VARCHAR(100) AS (UPPER(login)) STORED
    )",
    "CREATE TABLE orders (
        id INT AUTO_INCREMENT PRIMARY KEY,
        user_id INT NOT NULL,
        note VARCHAR(255),
        FOREIGN KEY (user_id) REFERENCES users(id)
    )",
    "CREATE TABLE audit (id INT AUTO_INCREMENT PRIMARY KEY, msg VARCHAR(255))",
    "CREATE TABLE skipped (id INT PRIMARY KEY)",
    "CREATE TRIGGER orders_ai AFTER INSERT ON orders FOR EACH ROW
        INSERT INTO audit(msg) VALUES (CONCAT('order ', NEW.id))",
    "CREATE VIEW user_emails AS SELECT id, email FROM users",
    "SET SESSION sql_mode = CONCAT(@@sql_mode, ',NO_AUTO_VALUE_ON_ZERO')",
    r#"INSERT INTO users (id, login, email, api_secret, bio, meta, balance, flags, created_at, born) VALUES
        (0, 'zero', 'zero@corp.com', UNHEX('00FF10'), NULL, '{"a": 1}', 0.00, b'000', '2024-01-01 10:00:00', NULL),
        (1, 'alice', 'alice@corp.com', UNHEX('DEADBEEF00'),
            'quote '' back\\slash\nnewline\ttab ctrlZ\Z nul\0 end', '{"k": "v\\"q"}', 12.50, b'101',
            '2024-02-02 11:11:11', '1990-05-06 07:08:09.123'),
        (2, 'bob', NULL, '', 'émoji 🎉 юникод', NULL, NULL, NULL, NULL, NULL)"#,
    "INSERT INTO orders (user_id, note) VALUES (1, 'o1'), (2, NULL), (0, 'x''y')",
    "INSERT INTO skipped VALUES (1)",
];

// all columns except anonymized ones (login, email, note)
const UNTOUCHED_QUERY: &str = "SELECT id, HEX(api_secret), HEX(bio), CAST(meta AS CHAR), \
    CAST(balance AS CHAR), flags + 0, CAST(created_at AS CHAR), CAST(born AS CHAR) \
    FROM users ORDER BY id";

const CONFIG: &str = r#"
filter:
  schema:
    except:
      - "*.skipped"
tables:
  - name: users
    rules:
      email:
        email: {}
      login:
        template:
          format: "user_{{ prev.id }}"
  - name: orders
    query:
      transform_condition: "note <> 'o1'"
    rules:
      note:
        template:
          format: "note-{{ prev.id }}"
"#;

fn is_configured() -> bool {
    env::var(DB_ENV_KEY).is_ok()
}

fn db_url(suffix: &str) -> Url {
    let mut url = Url::parse(&env::var(DB_ENV_KEY).unwrap()).expect("Invalid database URL");
    if !suffix.is_empty() {
        url.set_path(&format!("{}_{}", url.path(), suffix));
    }
    url
}

fn db_name(url: &Url) -> String {
    url.path().trim_start_matches('/').to_string()
}

fn recreate_db(url: &Url) -> Conn {
    let mut server_url = url.clone();
    server_url.set_path("");
    let mut conn = Conn::new(Opts::from_url(server_url.as_str()).unwrap()).unwrap();
    let name = db_name(url);
    conn.query_drop(format!("DROP DATABASE IF EXISTS `{}`", name))
        .unwrap();
    conn.query_drop(format!("CREATE DATABASE `{}` CHARACTER SET utf8mb4", name))
        .unwrap();
    conn.query_drop(format!("USE `{}`", name)).unwrap();
    conn
}

fn src_url() -> Url {
    CREATE_SRC_DB.call_once(|| {
        let mut conn = recreate_db(&db_url(""));
        conn.query_drop("SET NAMES utf8mb4").unwrap();
        for sql in SRC_SCHEMA {
            conn.query_drop(sql).unwrap();
        }
    });
    db_url("")
}

fn conn(url: &Url) -> Conn {
    let mut conn = Conn::new(Opts::from_url(url.as_str()).unwrap()).unwrap();
    conn.query_drop("SET NAMES utf8mb4").unwrap();
    conn
}

fn src_connection() -> Connection {
    Connector::new(src_url(), false, false).connect().unwrap()
}

fn dump(name: &str, config: &str, level: Option<IsolationLevel>) -> anyhow::Result<Url> {
    dump_from(&src_url(), name, config, level)
}

/// Dumps the `src` db with `config` and restores the dump into a new database via the `mysql` CLI
fn dump_from(
    src: &Url,
    name: &str,
    config: &str,
    level: Option<IsolationLevel>,
) -> anyhow::Result<Url> {
    let mut connection = Connector::new(src.clone(), false, false).connect().unwrap();
    let dst_url = db_url(name);
    recreate_db(&dst_url);

    let opts = Opts::from_url(dst_url.as_str()).unwrap();
    let mut cmd = Command::new(env::var(MYSQL_PATH_KEY).unwrap_or("mysql".to_string()));
    cmd.args([
        format!("--host={}", opts.get_ip_or_hostname()),
        format!("--port={}", opts.get_tcp_port()),
        "--protocol=TCP".to_string(),
        "--default-character-set=utf8mb4".to_string(),
        format!("--user={}", opts.get_user().unwrap_or_default()),
        db_name(&dst_url),
    ]);
    if let Some(pass) = opts.get_pass() {
        cmd.env("MYSQL_PWD", pass);
    }
    let mut restore = cmd.stdin(Stdio::piped()).spawn().unwrap();

    let engine = Engine::new(Settings::from_yaml(config).unwrap());
    let result = MyDumper::new(
        engine,
        level,
        env::var(MYSQLDUMP_PATH_KEY).unwrap_or("mysqldump".to_string()),
        restore.stdin.take().unwrap(),
        SilentIndicator,
        vec![],
    )
    .unwrap()
    .dump(&mut connection);

    assert!(restore.wait().unwrap().success(), "restore failed");
    result.map(|_| dst_url)
}

/// Values only: `Row` equality also compares column metadata (including the database name)
/// Dumps the source db into nowhere (for checking errors)
fn dump_to_sink(config: &str, mysqldump: String) -> anyhow::Result<()> {
    MyDumper::new(
        Engine::new(Settings::from_yaml(config).unwrap()),
        None,
        mysqldump,
        std::io::sink(),
        SilentIndicator,
        vec![],
    )
    .unwrap()
    .dump(&mut src_connection())
}

fn rows(conn: &mut Conn, sql: &str) -> Vec<Vec<Value>> {
    conn.query_map(sql, Row::unwrap).unwrap()
}

fn strings(conn: &mut Conn, sql: &str) -> Vec<Option<String>> {
    conn.query(sql).unwrap()
}

#[test]
fn dump_and_restore() {
    if !is_configured() {
        return;
    }

    let dst_url = dump("dump", CONFIG, Some(IsolationLevel::RepeatableRead)).unwrap();
    let mut src = conn(&src_url());
    let mut dst = conn(&dst_url);

    // untouched values (binary, JSON, escapes, unicode, NULL, id = 0...) are restored as is
    assert_eq!(
        rows(&mut src, UNTOUCHED_QUERY),
        rows(&mut dst, UNTOUCHED_QUERY)
    );

    // anonymized
    assert_eq!(
        strings(&mut dst, "SELECT login FROM users ORDER BY id"),
        vec![
            Some("user_0".into()),
            Some("user_1".into()),
            Some("user_2".into())
        ]
    );
    let emails = strings(&mut dst, "SELECT email FROM users WHERE id < 2 ORDER BY id");
    assert!(emails
        .iter()
        .all(|e| e.as_ref().is_some_and(|e| !e.contains("@corp.com"))));

    // generated column is recalculated
    assert_eq!(
        strings(&mut dst, "SELECT login_upper FROM users WHERE id = 1"),
        vec![Some("USER_1".into())]
    );

    // transform_condition: only matched rows are transformed, NULL stays NULL
    assert_eq!(
        strings(&mut dst, "SELECT note FROM orders ORDER BY id"),
        vec![Some("o1".into()), None, Some("note-3".into())]
    );

    // the trigger is restored, but didn't fire during data loading
    let audit = "SELECT msg FROM audit ORDER BY id";
    assert_eq!(strings(&mut src, audit), strings(&mut dst, audit));
    assert_eq!(
        strings(&mut dst, "SELECT TRIGGER_NAME FROM information_schema.TRIGGERS WHERE TRIGGER_SCHEMA = DATABASE()"),
        vec![Some("orders_ai".into())]
    );

    // the view is restored
    assert_eq!(
        strings(&mut dst, "SELECT COUNT(*) FROM user_emails"),
        vec![Some("3".into())]
    );

    // filtered out table
    assert!(strings(&mut dst, "SHOW TABLES LIKE 'skipped'").is_empty());
}

#[test]
fn dump_without_transaction() {
    if !is_configured() {
        return;
    }

    let dst_url = dump("no_tr", "tables: []", None).unwrap();
    let query = "SELECT * FROM orders ORDER BY id";
    assert_eq!(
        rows(&mut conn(&src_url()), query),
        rows(&mut conn(&dst_url), query)
    );
}

#[test]
fn asserts() {
    if !is_configured() {
        return;
    }

    let config = |count: u32| {
        format!(
            r#"
asserts:
  - name: no_negative_ids
    sql: SELECT id FROM users WHERE id < 0
    expect: no_rows
  - name: users_exist
    sql: SELECT id FROM users
    expect: rows_exist
  - name: user_count
    sql: SELECT COUNT(*) FROM users
    expect:
      eq: {}
  - name: string_scalar
    sql: SELECT login FROM users WHERE id = 1
    expect:
      eq: alice
  - name: null_scalar
    sql: SELECT email FROM users WHERE id = 2
    expect:
      not_eq: 1
  - name: only_a_warning
    sql: SELECT id FROM users
    expect: no_rows
    severity: warn
tables: []
"#,
            count
        )
    };

    assert!(dump("asserts_ok", &config(3), None).is_ok());

    let err = dump("asserts_fail", &config(4), None).unwrap_err();
    assert!(err.to_string().contains("user_count"));
    assert!(err.to_string().contains("got 3"));
}

#[test]
fn schema_inspector() {
    if !is_configured() {
        return;
    }

    let mut connection = src_connection();
    let tables = MySchemaInspector.ordered_tables(&mut connection).unwrap();
    let db = db_name(&src_url());

    let mut names: Vec<_> = tables.iter().map(|(t, _)| t.get_name()).collect();
    names.sort();
    // views are not tables
    assert_eq!(names, vec!["audit", "orders", "skipped", "users"]);

    let weight = |name: &str| tables.iter().find(|(t, _)| t.get_name() == name).unwrap();
    let users = weight("users");
    let orders = weight("orders");
    assert_eq!(users.0.get_full_name(), format!("{}.users", db));
    // `orders` depends on `users`, so `users` goes first
    assert!(users.1 > orders.1);
    // generated columns are skipped
    assert!(!users.0.get_columns_names().contains(&"login_upper".into()));

    let orders = &tables
        .iter()
        .find(|(t, _)| t.get_name() == "orders")
        .unwrap();
    assert_eq!(
        orders.0.get_dep_table_names(),
        vec![format!("{}.users", db)]
    );
}

/// Every MySQL column type (plus MariaDB-only ones), name => definition
const ALL_TYPES_COLUMNS: &[(&str, &str)] = &[
    ("c_tinyint", "TINYINT"),
    ("c_tinyint_u", "TINYINT UNSIGNED"),
    ("c_smallint", "SMALLINT"),
    ("c_mediumint", "MEDIUMINT"),
    ("c_int", "INT"),
    ("c_bigint", "BIGINT"),
    ("c_bigint_u", "BIGINT UNSIGNED"),
    ("c_decimal", "DECIMAL(65,30)"),
    ("c_float", "FLOAT"),
    ("c_double", "DOUBLE"),
    ("c_bit1", "BIT(1)"),
    ("c_bit64", "BIT(64)"),
    ("c_bool", "BOOLEAN"),
    ("c_date", "DATE"),
    ("c_datetime", "DATETIME(6)"),
    ("c_timestamp", "TIMESTAMP(6) NULL"),
    ("c_time", "TIME(6)"),
    ("c_year", "YEAR"),
    ("c_char", "CHAR(10)"),
    ("c_varchar", "VARCHAR(255)"),
    ("c_latin1", "VARCHAR(20) CHARACTER SET latin1"),
    ("c_binary", "BINARY(4)"),
    ("c_varbinary", "VARBINARY(16)"),
    ("c_tinyblob", "TINYBLOB"),
    ("c_blob", "BLOB"),
    ("c_mediumblob", "MEDIUMBLOB"),
    ("c_longblob", "LONGBLOB"),
    ("c_tinytext", "TINYTEXT"),
    ("c_text", "TEXT"),
    ("c_mediumtext", "MEDIUMTEXT"),
    ("c_longtext", "LONGTEXT"),
    ("c_enum", "ENUM('a', 'b c', '')"),
    ("c_set", "SET('x', 'y', 'z')"),
    ("c_json", "JSON"),
    ("c_geometry", "GEOMETRY"),
    ("c_point", "POINT"),
    ("c_linestring", "LINESTRING"),
    ("c_polygon", "POLYGON"),
    ("c_multipoint", "MULTIPOINT"),
    ("c_multilinestring", "MULTILINESTRING"),
    ("c_multipolygon", "MULTIPOLYGON"),
    ("c_geometrycollection", "GEOMETRYCOLLECTION"),
];

const MARIADB_TYPES_COLUMNS: &[(&str, &str)] = &[("c_inet6", "INET6"), ("c_uuid", "UUID")];

/// max, min, zero/empty values (the 4th row is all NULLs)
const ALL_TYPES_ROWS: &[&[&str]] = &[
    &[
        "127",
        "255",
        "32767",
        "8388607",
        "2147483647",
        "9223372036854775807",
        "18446744073709551615",
        "'99999999999999999999999999999999999.999999999999999999999999999999'",
        "3.40282e38",
        "1.7976931348623157e308",
        "b'1'",
        "0xFFFFFFFFFFFFFFFF",
        "TRUE",
        "'9999-12-31'",
        "'9999-12-31 23:59:59.999999'",
        "'2038-01-19 03:14:07.999999'",
        "'838:59:59.000000'",
        "2155",
        "'abc'",
        r"'émoji 🎉 юникод '' \\ \n \r \t \0 \Z %_'",
        "'café'",
        "0x00275C0A",
        "0x00FF275C1A0D0A",
        "0x00",
        "UNHEX(REPEAT('00FF275C1A0D0A22', 1000))",
        // ~1.6MB: more than the INSERT batch size
        "UNHEX(REPEAT('00FF275C1A0D0A22', 200000))",
        "UNHEX(REPEAT('5C', 100))",
        "'tiny'",
        r"'multi\nline\ttext'",
        "REPEAT('ж', 10000)",
        "REPEAT('🎉', 300000)",
        "'b c'",
        "'x,z'",
        r#"'{"a": [1, 2.5, null, true], "s": "q\\"\\\\ 🎉"}'"#,
        "ST_GeomFromText('POINT(1 2)')",
        "ST_GeomFromText('POINT(-1.5 2.25)')",
        "ST_GeomFromText('LINESTRING(0 0, 1 1, 2 2)')",
        "ST_GeomFromText('POLYGON((0 0, 10 0, 10 10, 0 10, 0 0), (1 1, 2 1, 2 2, 1 1))')",
        "ST_GeomFromText('MULTIPOINT(0 0, 1 1)')",
        "ST_GeomFromText('MULTILINESTRING((0 0, 1 1), (2 2, 3 3))')",
        "ST_GeomFromText('MULTIPOLYGON(((0 0, 1 0, 1 1, 0 0)))')",
        "ST_GeomFromText('GEOMETRYCOLLECTION(POINT(1 1), LINESTRING(0 0, 1 1))')",
    ],
    &[
        "-128",
        "0",
        "-32768",
        "-8388608",
        "-2147483648",
        "-9223372036854775808",
        "0",
        "'-99999999999999999999999999999999999.999999999999999999999999999999'",
        "-3.40282e38",
        "-1.7976931348623157e308",
        "b'0'",
        "0x00",
        "FALSE",
        "'1000-01-01'",
        "'1000-01-01 00:00:00.000000'",
        "'1970-01-01 00:00:01.000000'",
        "'-838:59:59.000000'",
        "1901",
        "''",
        "''",
        "''",
        "0x00000000",
        "''",
        "''",
        "''",
        "''",
        "''",
        "''",
        "''",
        "''",
        "''",
        "''",
        "''",
        "'[]'",
        "ST_GeomFromText('POINT(0 0)')",
        "ST_GeomFromText('POINT(0 0)')",
        "ST_GeomFromText('LINESTRING(0 0, 0 1)')",
        "ST_GeomFromText('POLYGON((0 0, 1 0, 1 1, 0 0))')",
        "ST_GeomFromText('MULTIPOINT(0 0)')",
        "ST_GeomFromText('MULTILINESTRING((0 0, 1 1))')",
        "ST_GeomFromText('MULTIPOLYGON(((0 0, 1 0, 1 1, 0 0)))')",
        "ST_GeomFromText('GEOMETRYCOLLECTION EMPTY')",
    ],
    &[
        "0",
        "1",
        "0",
        "0",
        "0",
        "0",
        "1",
        "'0.000000000000000000000000000001'",
        "1.17549e-38",
        "2.2250738585072014e-308",
        "b'1'",
        "0x0000000000000001",
        "2",
        "'0000-00-00'",
        "'0000-00-00 00:00:00.000000'",
        "'2024-02-29 12:00:00.000001'",
        "'00:00:00.000001'",
        "0",
        "' x'",
        "'  trailing  '",
        "'ÿ'",
        "0x01",
        "0x01",
        "0x01",
        "0x01",
        "0x01",
        "0x01",
        "' '",
        "' '",
        "' '",
        "' '",
        "'a'",
        "'x,y,z'",
        "'\"str\"'",
        "ST_GeomFromText('POINT(1e10 -1e-10)')",
        "ST_GeomFromText('POINT(0.1 0.2)')",
        "ST_GeomFromText('LINESTRING(0.1 0.2, 0.3 0.4)')",
        "ST_GeomFromText('POLYGON((0 0, 1 0, 1 1, 0 0))')",
        "ST_GeomFromText('MULTIPOINT(1 1)')",
        "ST_GeomFromText('MULTILINESTRING((0 0, 1 1))')",
        "ST_GeomFromText('MULTIPOLYGON(((0 0, 1 0, 1 1, 0 0)))')",
        "ST_GeomFromText('GEOMETRYCOLLECTION(POINT(0 0))')",
    ],
];

const MARIADB_ROWS: &[&[&str]] = &[
    &[
        "'ffff:ffff:ffff:ffff:ffff:ffff:ffff:ffff'",
        "'ffffffff-ffff-ffff-ffff-ffffffffffff'",
    ],
    &["'::'", "'00000000-0000-0000-0000-000000000000'"],
    &["'::ffff:1.2.3.4'", "'123e4567-e89b-12d3-a456-426614174000'"],
];

/// Types written as hex literals: a transformer result is inserted as a string into them
fn is_binary_type(definition: &str) -> bool {
    [
        "BINARY", "BLOB", "BIT", "GEOM", "POINT", "STRING", "POLYGON",
    ]
    .iter()
    .any(|t| definition.contains(t))
}

#[test]
fn all_types() {
    if !is_configured() {
        return;
    }

    let src = db_url("types_src");
    let mut setup = recreate_db(&src);
    let version: String = setup.query_first("SELECT VERSION()").unwrap().unwrap();
    let is_mariadb = version.contains("MariaDB");

    let mut columns: Vec<(&str, &str)> = ALL_TYPES_COLUMNS.to_vec();
    let mut values: Vec<Vec<&str>> = ALL_TYPES_ROWS.iter().map(|r| r.to_vec()).collect();
    if is_mariadb {
        columns.extend(MARIADB_TYPES_COLUMNS);
        for (row, extra) in values.iter_mut().zip(MARIADB_ROWS) {
            row.extend(extra.iter());
        }
    }
    let names: Vec<&str> = columns.iter().map(|(n, _)| *n).collect();

    setup.query_drop("SET NAMES utf8mb4").unwrap();
    // allow zero dates
    setup
        .query_drop("SET SESSION sql_mode = 'NO_AUTO_VALUE_ON_ZERO'")
        .unwrap();
    setup
        .query_drop(format!(
            "CREATE TABLE all_types (id INT PRIMARY KEY, {})",
            columns
                .iter()
                .map(|(n, d)| format!("{} {}", n, d))
                .collect::<Vec<_>>()
                .join(", ")
        ))
        .unwrap();
    setup
        .query_drop("CREATE TABLE all_types_tr LIKE all_types")
        .unwrap();
    for (i, row) in values.iter().enumerate() {
        assert_eq!(row.len(), names.len(), "row {}", i + 1);
        setup
            .query_drop(format!(
                "INSERT INTO all_types (id, {}) VALUES ({}, {})",
                names.join(", "),
                i + 1,
                row.join(", ")
            ))
            .unwrap();
    }
    setup
        .query_drop("INSERT INTO all_types (id) VALUES (100)")
        .unwrap();
    setup
        .query_drop("INSERT INTO all_types_tr SELECT * FROM all_types")
        .unwrap();

    // `all_types_tr`: every non-binary value goes through the transform engine and back
    let rules: String = columns
        .iter()
        .filter(|(_, d)| !is_binary_type(d))
        .map(|(n, _)| {
            format!("      {n}:\n        template:\n          format: \"{{{{ prev.{n} }}}}\"\n")
        })
        .collect();
    let config = format!("tables:\n  - name: all_types_tr\n    rules:\n{}", rules);
    let dst = dump_from(
        &src,
        "types_dst",
        &config,
        Some(IsolationLevel::RepeatableRead),
    )
    .unwrap();

    let mut src_conn = conn(&src);
    let mut dst_conn = conn(&dst);
    for table in ["all_types", "all_types_tr"] {
        let query = format!("SELECT * FROM {} ORDER BY id", table);
        let expected = rows(&mut src_conn, &query);
        let actual = rows(&mut dst_conn, &query);
        assert_eq!(expected.len(), 4);
        for (e, a) in expected.iter().zip(&actual) {
            for (i, name) in std::iter::once("id")
                .chain(names.iter().copied())
                .enumerate()
            {
                assert_eq!(e[i], a[i], "{}.{} (row id {:?})", table, name, e[0]);
            }
        }
    }
}

#[test]
fn mysqldump_error() {
    if !is_configured() {
        return;
    }

    // `false` exits with an error
    let err = dump_to_sink("tables: []", "false".to_string()).unwrap_err();
    assert!(err.to_string().contains("mysqldump error"));
}

#[test]
fn transform_error() {
    if !is_configured() {
        return;
    }

    let config = r#"
tables:
  - name: users
    rules:
      email:
        template:
          format: "{{ prev.no_such_column }}"
"#;
    let mysqldump = env::var(MYSQLDUMP_PATH_KEY).unwrap_or("mysqldump".to_string());
    assert!(dump_to_sink(config, mysqldump).is_err());
}

#[test]
fn only_filter() {
    if !is_configured() {
        return;
    }

    let config = "filter:\n  schema:\n    only:\n      - \"*.audit\"\ntables: []";
    let dst_url = dump("only", config, None).unwrap();
    assert_eq!(
        strings(&mut conn(&dst_url), "SHOW TABLES"),
        vec![Some("audit".into())]
    );
}

#[test]
fn tls() {
    if !is_configured() {
        return;
    }

    let cipher = |mut c: Connection| -> String {
        let row: Option<(String, String)> = c
            .conn
            .query_first("SHOW SESSION STATUS LIKE 'Ssl_cipher'")
            .unwrap();
        row.unwrap().1
    };

    // the test server uses a self-signed certificate
    let with_flags = Connector::new(src_url(), true, true).connect().unwrap();
    assert!(!cipher(with_flags).is_empty());

    let mut url = src_url();
    url.set_query(Some(
        "danger_accept_invalid_certs=true&danger_skip_domain_validation=true",
    ));
    let with_params = Connector::new(url, false, false).connect().unwrap();
    assert!(!cipher(with_params).is_empty());

    let plain = Connector::new(src_url(), false, false).connect().unwrap();
    assert!(cipher(plain).is_empty());
}
