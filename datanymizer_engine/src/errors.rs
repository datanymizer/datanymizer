use thiserror::Error;

use crate::transformer::TransformError;

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct UnknownColumnError {
    pub field_name: String,
}

#[derive(Error, Debug)]
pub enum EngineError {
    #[error("Failed transform {0}")]
    TransformFieldError(TransformError),
    #[error("Unknown column")]
    UnknownColumnError(UnknownColumnError),
}
