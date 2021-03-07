use super::helpers;

use datanymizer_dumper::{indicator::SilentIndicator, Dumper};
use datanymizer_engine::{Engine, Settings};
use pg_datanymizer_dumper::{connector::Connection, dumper::PgDumper};

fn dump(name: &str) {
    let mut dst = helpers::dst_wrapper(name);

    let cfg_filename = format!("tests/configs/{}.yml", name);
    let settings = Settings::new(cfg_filename).unwrap();
    let engine = Engine::new(settings);
    let mut dumper = PgDumper::new(
        engine,
        None,
        helpers::pg_dump_path(),
        dst.io(),
        SilentIndicator,
        vec![],
    )
    .unwrap();
    let mut connection = Connection::new(helpers::src_client(), helpers::src_database_url());
    dumper.dump(&mut connection).unwrap();

    dst.close();
}

#[test]
fn simple_dump() {
    dump("simple");

    let mut src_client = helpers::src_client();
    let mut dst_client = helpers::dst_client("simple");

    let count_query = "SELECT COUNT(*) FROM actor";
    let src_count: i64 = src_client.query_one(count_query, &[]).unwrap().get(0);
    let dst_count: i64 = dst_client.query_one(count_query, &[]).unwrap().get(0);
    assert_eq!(src_count, dst_count);

    let rows_query = "SELECT * FROM actor";
    let src_rows = src_client.query(rows_query, &[]).unwrap();
    let dst_rows = dst_client.query(rows_query, &[]).unwrap();
    for (i, src_row) in src_rows.iter().enumerate() {
        let dst_row = &dst_rows[i];

        let src_actor_id: i32 = src_row.get("actor_id");
        let dst_actor_id: i32 = dst_row.get("actor_id");
        assert_eq!(src_actor_id, dst_actor_id);

        let src_first_name: String = src_row.get("first_name");
        let dst_first_name: String = dst_row.get("first_name");
        let src_last_name: String = src_row.get("last_name");
        let dst_last_name: String = dst_row.get("last_name");
        assert_ne!(
            format!("{} {}", src_first_name, src_last_name),
            format!("{} {}", dst_first_name, dst_last_name)
        );
    }
}
