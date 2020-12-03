use thiserror::Error;

use crate::transformer::TransformError;

#[derive(Error, Debug)]
pub enum EngineError {
    #[error("Transform for `{0}` is failed")]
    TransformFieldError(TransformError),
}
