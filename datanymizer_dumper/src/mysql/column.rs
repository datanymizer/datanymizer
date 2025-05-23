use crate::ColumnData;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct MysqlColumn {
    name: String,
    position: usize,
    inner_type: String,
}

impl MysqlColumn {
    pub fn new(name: String, position: usize, inner_type: String) -> Self {
        Self {
            name,
            position,
            inner_type,
        }
    }
}

impl ColumnData<String> for MysqlColumn {
    fn position(&self) -> usize {
        self.position
    }

    fn name(&self) -> &str {
        self.name.as_str()
    }

    fn inner_kind(&self) -> Option<String> {
        Some(self.inner_type.clone())
    }
}
