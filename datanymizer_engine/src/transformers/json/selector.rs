use jsonpath_lib::JsonPathError;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq, Hash)]
#[serde(from = "String")]
pub struct Selector {
    path: String,
}

impl Selector {
    pub fn replace<F: FnMut(Value) -> Option<Value>>(
        &self,
        value: Value,
        f: &mut F,
    ) -> Result<Value, JsonPathError> {
        jsonpath_lib::replace_with(value, self.path.as_str(), f)
    }
}

impl From<String> for Selector {
    fn from(path: String) -> Self {
        Self { path }
    }
}
