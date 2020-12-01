use crate::ColumnData;
use postgres::{types::Type, Row as PostgresRow};
use std::cmp::Ordering;

#[derive(Debug, Clone, Eq)]
pub struct PgColumn {
    /// Ordinal position of column
    pub position: i32,
    /// Column name
    pub name: String,
    /// Column data type
    pub data_type: String,

    /// Inner postgres type (oid)
    pub inner_type: Option<u32>,
}

impl PartialEq for PgColumn {
    fn eq(&self, other: &Self) -> bool {
        self.position == other.position
    }
}

impl PartialOrd for PgColumn {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for PgColumn {
    fn cmp(&self, other: &Self) -> Ordering {
        self.position.cmp(&other.position)
    }
}

impl From<PostgresRow> for PgColumn {
    fn from(row: PostgresRow) -> Self {
        let oid: u32 = row.get("oid");

        Self {
            position: row.get("ordinal_position"),
            name: row.get("column_name"),
            data_type: row.get("data_type"),
            inner_type: Some(oid),
        }
    }
}

impl ColumnData<Type> for PgColumn {
    fn position(&self) -> usize {
        (self.position - 1) as usize
    }

    fn name(&self) -> &str {
        self.name.as_ref()
    }

    fn inner_kind(&self) -> Option<Type> {
        match self.inner_type {
            Some(oid) => Type::from_oid(oid),
            None => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cmp::Ordering;

    #[test]
    fn ordering_test() {
        let col1 = &PgColumn {
            position: 1,
            name: String::from("Column1"),
            data_type: String::new(),
            inner_type: Some(0),
        };
        let col2 = &PgColumn {
            position: 2,
            name: String::from("Column2"),
            data_type: String::new(),
            inner_type: Some(0),
        };

        let col3 = &PgColumn {
            position: 1,
            name: String::from("Column1"),
            data_type: String::new(),
            inner_type: Some(0),
        };

        assert_eq!(col1, col3);
        assert_eq!(col1.cmp(col2), Ordering::Less);
        assert_eq!(col1.cmp(col3), Ordering::Equal);
    }
}
