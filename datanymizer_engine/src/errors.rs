use thiserror::Error;

use crate::transformer::TransformError;
use std::fmt::{self, Display, Formatter};

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct UnknownColumnError {
    pub field_name: String,
}

impl Display for UnknownColumnError {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        write!(formatter, "{}", self.field_name)
    }
}

#[derive(Error, Debug)]
pub enum EngineError {
    #[error("Failed transform {0}")]
    TransformFieldError(TransformError),
    #[error("Unknown column {0}")]
    UnknownColumnError(UnknownColumnError),
}
