use super::helpers;

use datanymizer_dumper::postgres::asserts::run_asserts;
use datanymizer_engine::Settings;

fn run(yaml: &str) -> anyhow::Result<()> {
    let settings = Settings::from_yaml(yaml).unwrap();
    let mut client = helpers::src_client();
    run_asserts(&settings, &mut client, None)
}

fn actor_count() -> i64 {
    let mut client = helpers::src_client();
    client
        .query_one("select count(*) from actor", &[])
        .unwrap()
        .get(0)
}

#[test]
fn no_rows_passes() {
    if !helpers::is_configured() {
        return;
    }

    let result = run(r#"
asserts:
  - name: actor_absent
    sql: select actor_id from actor where actor_id < 0
    expect: no_rows
tables: []
"#);

    assert!(result.is_ok());
}

#[test]
fn no_rows_fails() {
    if !helpers::is_configured() {
        return;
    }

    let result = run(r#"
asserts:
  - name: actor_present
    sql: select actor_id from actor limit 1
    expect: no_rows
tables: []
"#);

    let error = result.unwrap_err().to_string();
    assert!(error.contains("actor_present"));
    assert!(error.contains("expected no rows"));
}

#[test]
fn eq_passes() {
    if !helpers::is_configured() {
        return;
    }

    let result = run(format!(
        r#"
asserts:
  - name: actor_count
    sql: select count(*) from actor
    expect:
      eq: {}
tables: []
"#,
        actor_count()
    )
    .as_str());

    assert!(result.is_ok());
}

#[test]
fn rows_exist_passes() {
    if !helpers::is_configured() {
        return;
    }

    let result = run(r#"
asserts:
  - name: actor_rows_exist
    sql: select actor_id from actor limit 1
    expect: rows_exist
tables: []
"#);

    assert!(result.is_ok());
}

#[test]
fn scalar_range_passes() {
    if !helpers::is_configured() {
        return;
    }

    let count = actor_count();
    let result = run(format!(
        r#"
asserts:
  - name: actor_count_in_range
    sql: select count(*) from actor
    expect:
      gt: {}
      lt: {}
tables: []
"#,
        count - 1,
        count + 1
    )
    .as_str());

    assert!(result.is_ok());
}

#[test]
fn warn_does_not_fail() {
    if !helpers::is_configured() {
        return;
    }

    let result = run(r#"
asserts:
  - name: impossible_actor_count
    sql: select count(*) from actor
    expect:
      eq: 0
    severity: warn
tables: []
"#);

    assert!(result.is_ok());
}

#[test]
fn table_level_asserts_are_executed() {
    if !helpers::is_configured() {
        return;
    }

    let result = run(format!(
        r#"
tables:
  - name: actor
    rules: {{}}
    asserts:
      - name: actor_rows_exist
        sql: select count(*) from actor
        expect:
          eq: {}
"#,
        actor_count()
    )
    .as_str());

    assert!(result.is_ok());
}
