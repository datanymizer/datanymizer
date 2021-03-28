use super::sql_type::MsSqlType;
use crate::ColumnData;
use std::cmp::Ordering;
use tiberius::Row;

#[derive(Debug, Clone, Eq)]
pub struct MsSqlColumn {
    /// Ordinal position of column
    pub position: i32,
    /// Column name
    pub name: String,
    /// Column type
    pub data_type: MsSqlType,
}

impl MsSqlColumn {
    pub fn expression_for_query_from(&self) -> String {
        if self.data_type.has_supported_type() {
            format!("[{}]", self.name)
        } else {
            format!("CAST([{}] AS varbinary)", self.name)
        }
    }
}

impl PartialEq for MsSqlColumn {
    fn eq(&self, other: &Self) -> bool {
        self.position == other.position
    }
}

impl PartialOrd for MsSqlColumn {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for MsSqlColumn {
    fn cmp(&self, other: &Self) -> Ordering {
        self.position.cmp(&other.position)
    }
}

impl ColumnData<MsSqlType> for MsSqlColumn {
    fn position(&self) -> usize {
        (self.position - 1) as usize
    }

    fn name(&self) -> &str {
        self.name.as_ref()
    }

    fn inner_kind(&self) -> Option<MsSqlType> {
        Some(self.data_type.clone())
    }
}

impl From<&Row> for MsSqlColumn {
    fn from(row: &Row) -> Self {
        let name = row
            .get::<&str, _>("COLUMN_NAME")
            .expect("table name column is missed")
            .to_string();
        let position = row
            .get::<i32, _>("ORDINAL_POSITION")
            .expect("position column is missed");
        let data_type = MsSqlType(
            row.get::<&str, _>("DATA_TYPE")
                .expect("type column is missed")
                .to_string(),
        );

        Self {
            position,
            name,
            data_type,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
}
