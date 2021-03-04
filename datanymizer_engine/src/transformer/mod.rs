mod context;
mod uniq_transformer;
mod uniqueness;

pub use context::TransformContext;
pub use uniq_transformer::UniqTransformer;
pub use uniqueness::Uniqueness;

use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;
use std::fmt::{self, Display, Formatter};
use std::{error, result};

use crate::LocaleConfig;

pub type TransformResult = result::Result<Option<String>, TransformError>;
pub type Globals = HashMap<String, Value>;

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct TransformError {
    pub field_name: String,
    pub field_value: String,
    pub reason: String,
}

#[derive(Debug, Default, Deserialize, Clone)]
#[serde(default)]
pub struct TransformerDefaults {
    pub locale: LocaleConfig,
}

pub trait TransformResultHelper {
    fn present<T>(value: T) -> TransformResult
    where
        T: ToString,
    {
        Ok(Some(value.to_string()))
    }

    fn error<T>(field_name: T, field_value: T, reason: T) -> TransformResult
    where
        T: ToString,
    {
        Err(TransformError {
            field_name: field_name.to_string(),
            field_value: field_value.to_string(),
            reason: reason.to_string(),
        })
    }
}

impl TransformResultHelper for TransformResult {}

impl Display for TransformError {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        write!(formatter, "{}", self.reason)
    }
}

pub trait Transformer {
    fn transform(
        &self,
        field_name: &str,
        field_value: &str,
        ctx: &Option<TransformContext>,
    ) -> TransformResult;

    fn set_defaults(&mut self, _defaults: &TransformerDefaults) {}
}

impl error::Error for TransformError {
    fn description(&self) -> &str {
        &self.reason
    }
}

impl From<tera::Error> for TransformError {
    fn from(err: tera::Error) -> Self {
        Self {
            field_name: "".to_string(),
            field_value: "".to_string(),
            reason: err.to_string(),
        }
    }
}

impl From<chrono::ParseError> for TransformError {
    fn from(pe: chrono::ParseError) -> Self {
        Self {
            field_name: "".to_string(),
            field_value: "".to_string(),
            reason: pe.to_string(),
        }
    }
}
