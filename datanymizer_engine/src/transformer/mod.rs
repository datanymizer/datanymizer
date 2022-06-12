mod context;
mod uniq_transformer;
mod uniqueness;

pub use context::TransformContext;
pub use uniq_transformer::UniqTransformer;
pub use uniqueness::Uniqueness;

use serde::Deserialize;
use serde_json::Value;
use std::{
    collections::HashMap,
    error,
    fmt::{self, Display, Formatter},
    sync::{Arc, RwLock},
};

use crate::{settings::TemplatesCollection, LocaleConfig};

pub type TransformResult = Result<Option<String>, TransformError>;
pub type Globals = HashMap<String, Value>;
type TemplateStore = Arc<RwLock<HashMap<String, tera::Value>>>;

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

#[derive(Default)]
pub struct TransformerInitContext {
    pub defaults: TransformerDefaults,
    pub template_store: TemplateStore,
    pub template_collection: TemplatesCollection,
}

impl TransformerInitContext {
    pub fn from_defaults(defaults: TransformerDefaults) -> Self {
        Self {
            defaults,
            template_store: TemplateStore::default(),
            template_collection: TemplatesCollection::default(),
        }
    }
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

    fn init(&mut self, _ctx: &TransformerInitContext) {}
}

impl error::Error for TransformError {
    fn description(&self) -> &str {
        &self.reason
    }
}

impl TransformError {
    fn from_error<E: error::Error>(err: E) -> Self {
        Self {
            field_name: "".to_string(),
            field_value: "".to_string(),
            reason: err.to_string(),
        }
    }
}

impl From<tera::Error> for TransformError {
    fn from(err: tera::Error) -> Self {
        Self::from_error(err)
    }
}

impl From<time::error::Format> for TransformError {
    fn from(err: time::error::Format) -> Self {
        Self::from_error(err)
    }
}

impl From<time::error::InvalidFormatDescription> for TransformError {
    fn from(err: time::error::InvalidFormatDescription) -> Self {
        Self::from_error(err)
    }
}
