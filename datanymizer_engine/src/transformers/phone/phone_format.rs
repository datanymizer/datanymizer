use serde::{Deserialize, Serialize};
use std::cmp;

const DEFAULT_FORMAT: &str = "+###########";

#[derive(Serialize, Deserialize, PartialEq, Eq, Hash, Debug, Clone)]
pub struct PhoneFormat {
    pub source_format: String,
    pub invariants_len: u32,
}

impl PhoneFormat {
    pub fn from_format(format: String) -> Self {
        Self {
            source_format: format.clone(),
            invariants_len: format.matches(&['#', '^'][..]).count() as u32,
        }
    }
}

impl PhoneFormat {
    pub fn invariants(&self) -> i64 {
        10i64.pow(cmp::min(self.invariants_len, 10u32))
    }
}

impl Default for PhoneFormat {
    fn default() -> Self {
        Self::from_format(DEFAULT_FORMAT.to_string())
    }
}
