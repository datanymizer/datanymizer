use super::helpers;

use datanymizer_dumper::{
    postgres::{schema_inspector::PgSchemaInspector, table::PgTable},
    SchemaInspector, Table,
};

fn find_table<'a>(tables: &'a [PgTable], full_name: &str) -> &'a PgTable {
    tables
        .iter()
        .find(|t| t.get_full_name() == full_name)
        .unwrap()
}

#[test]
fn get_tables() {
    let mut connection = helpers::src_connection();
    let inspector = PgSchemaInspector;
    let tables = inspector.get_tables(&mut connection).unwrap();

    let table = find_table(&tables, "public.actor");
    assert_eq!(table.tablename, "actor");
    assert_eq!(table.schemaname, "public");
}
