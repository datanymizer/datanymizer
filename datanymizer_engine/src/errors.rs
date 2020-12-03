use thiserror::Error;

use crate::transformer::TransformError;

#[derive(Error, Debug)]
pub enum EngineError {
    #[error("Failed transform {0}")]
    TransformFieldError(TransformError),
}
