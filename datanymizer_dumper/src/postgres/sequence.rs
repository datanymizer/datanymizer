#[derive(Debug, Clone, Eq, PartialEq)]
pub struct PgSequence {
    pub full_name: String,
}

impl PgSequence {
    pub fn setval_query(&self, last_value: i64) -> String {
        format!(
            "SELECT pg_catalog.setval('{}', {}, true);",
            self.full_name, last_value
        )
    }

    pub fn last_value_query(&self) -> String {
        format!("SELECT last_value FROM {}", self.full_name)
    }
}
