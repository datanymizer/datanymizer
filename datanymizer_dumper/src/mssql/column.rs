use super::MsSqlType;
use crate::ColumnData;
use std::cmp::Ordering;

#[derive(Debug, Clone, Eq)]
pub struct MsSqlColumn {
    /// Ordinal position of column
    pub position: i32,
    /// Column name
    pub name: String,
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
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
}
