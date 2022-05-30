use super::helpers;

use datanymizer_dumper::{
    postgres::{connector::Connection, schema_inspector::PgSchemaInspector, table::PgTable},
    SchemaInspector, Table,
};

fn find_table<'a>(tables: &'a [PgTable], full_name: &str) -> &'a PgTable {
    tables
        .iter()
        .find(|t| t.get_full_name() == full_name)
        .unwrap()
}

fn assert_cols(table: &PgTable, expected: Vec<&'static str>) {
    let mut cols: Vec<_> = table
        .columns
        .iter()
        .map(|c| (c.position, c.name.clone()))
        .collect();
    cols.sort_by_key(|(p, _)| *p);
    let cols: Vec<_> = cols.iter().map(|(_, name)| name.clone()).collect();
    assert_eq!(cols, expected);
}

fn assert_fks(table: &PgTable, expected: Vec<&'static str>) {
    let deps = table.get_dep_table_names();
    for name in expected {
        assert!(deps.contains(&name.to_string()));
    }
}

#[test]
fn get_tables() {
    let mut connection = Connection::new(helpers::src_client(), helpers::src_database_url());
    let inspector = PgSchemaInspector;
    let tables = inspector.get_tables(&mut connection).unwrap();

    let table = find_table(&tables, "public.actor");
    assert_eq!(table.tablename, "actor");
    assert_eq!(table.schemaname, "public");
    assert_cols(table, vec!["actor_id", "first_name", "last_name", "last_update"]);
    assert_fks(table, vec![]);

    let table = find_table(&tables, "public.address");
    assert_eq!(table.tablename, "address");
    assert_eq!(table.schemaname, "public");
    assert_cols(
        table,
        vec![
            "address_id",
            "address",
            "address2",
            "district",
            "city_id",
            "postal_code",
            "phone",
            "last_update"
        ]
    );
    assert_fks(table, vec!["public.city"]);

    let table = find_table(&tables, "public.payment");
    assert_eq!(table.tablename, "payment");
    assert_eq!(table.schemaname, "public");
    assert_cols(
        table,
        vec![
            "payment_id",
            "customer_id",
            "staff_id",
            "rental_id",
            "amount",
            "payment_date"
        ]
    );
    assert_fks(table, vec!["public.customer", "public.rental", "public.staff"]);
}
