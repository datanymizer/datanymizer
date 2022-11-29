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

impl From<&str> for Selector {
    fn from(path: &str) -> Self {
        Self {
            path: path.to_string(),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use serde_json::json;

    fn replace_name(_: Value) -> Option<Value> {
        Some(Value::String("John".to_string()))
    }

    #[test]
    fn strict_path() {
        let selector = Selector::from("$.user.personal.name");
        let json = json!(
            { "user": { "personal": { "name": "Andrew" }, "age": 20 } }
        );

        let result = selector.replace(json, &mut replace_name).unwrap();
        assert_eq!(
            result,
            json!(
                { "user": { "personal": { "name": "John" }, "age": 20 } }
            )
        );
    }

    #[test]
    fn any_path() {
        let selector = Selector::from("$..user.name");
        let json = json!(
            [
                { "user": { "name": "Andrew", "age": 20 } },
                { "user": { "name": "Briana", "age": 20 } },
                { "user": { "name": "Charlie", "age": 20 } }
            ]
        );

        let result = selector.replace(json, &mut replace_name).unwrap();
        assert_eq!(
            result,
            json!(
                [
                    { "user": { "name": "John", "age": 20 } },
                    { "user": { "name": "John", "age": 20 } },
                    { "user": { "name": "John", "age": 20 } }
                ]
            )
        );
    }

    #[test]
    fn index() {
        let selector = Selector::from("$[0, 2].user.name");
        let json = json!(
            [
                { "user": { "name": "Andrew", "age": 40 } },
                { "user": { "name": "Briana", "age": 20 } },
                { "user": { "name": "Charlie", "age": 20 } }
            ]
        );

        let result = selector.replace(json, &mut replace_name).unwrap();
        assert_eq!(
            result,
            json!(
                [
                  { "user": { "name": "John", "age": 40 } },
                  { "user": { "name": "Briana", "age": 20 } },
                  { "user": { "name": "John", "age": 20 } }
                ]
            )
        );
    }

    #[test]
    fn filter() {
        let selector = Selector::from("$..user[?(@.age > 30)].name");
        let json = json!(
            [
                { "user": { "name": "Andrew", "age": 20 } },
                { "user": { "name": "Briana", "age": 30 } },
                { "user": { "name": "Charlie", "age": 40 } }
            ]
        );

        let result = selector.replace(json, &mut replace_name).unwrap();
        assert_eq!(
            result,
            json!(
                [
                { "user": { "name": "Andrew", "age": 20 } },
                { "user": { "name": "Briana", "age": 30 } },
                { "user": { "name": "John", "age": 40 } }
                ]
            )
        );
    }
}
