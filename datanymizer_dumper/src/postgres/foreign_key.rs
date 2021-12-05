use postgres::Row as PostgresRow;

#[derive(Debug)]
pub struct ForeignKey {
    // Source
    pub table_schema: String,
    pub table_name: String,
    pub constraint_name: String,

    pub column_name: String,

    // Reference
    pub foreign_table_schema: String,
    pub foreign_table_name: String,
    pub foreign_column_name: String,
}

impl From<PostgresRow> for ForeignKey {
    fn from(row: PostgresRow) -> Self {
        Self {
            table_schema: row.get("table_schema"),
            table_name: row.get("table_name"),
            constraint_name: row.get("constraint_name"),

            column_name: row.get("column_name"),

            // Reference
            foreign_table_schema: row.get("foreign_table_schema"),
            foreign_table_name: row.get("foreign_table_name"),
            foreign_column_name: row.get("foreign_column_name"),
        }
    }
}
