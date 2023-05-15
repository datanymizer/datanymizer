use crate::{
    transformer::TransformResultHelper, TransformContext, TransformResult, Transformer,
    TransformerInitContext, Transformers,
};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::Value;

mod selector;
use selector::Selector;

/// This transformer allows to replace values in JSON and JSONB columns using JSONPath selectors.
/// It uses the [jsonpath_lib](https://github.com/freestrings/jsonpath) crate.
///
/// # Example:
///
/// ```yaml
/// #...
/// rules:
///   fields:
///     - name: "user_name"
///       selector: "$..user.name"
///       quote: true
///       rule:
///         person_name: {}
///     - name: "user_age"
///       selector: "$..user.age"
///       rule:
///         random_num:
///           min: 25
///           max: 55
/// ```
///
/// If a value of the column is `{"user": {"name": "Andrew", "age": 20, "comment": "The comment"}}`,
/// the transformed value will be something like this:
/// `{"user": {"name": "John", "age": 30, "comment": "The comment"}}`.
#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq, Hash)]
pub struct JsonTransformer {
    fields: Vec<Field>,
    #[serde(default, with = "serde_yaml::with::singleton_map")]
    on_invalid: OnInvalid,
}

impl JsonTransformer {
    fn transform_parsed_value(
        &self,
        field_name: &str,
        mut value: Value,
        ctx: &Option<TransformContext>,
    ) -> Result<String> {
        let mut err: Option<anyhow::Error> = None;
        for field in &self.fields {
            let replace_result = field.selector.replace(value, &mut |v| {
                let transform_result =
                    field
                        .rule
                        .transform(field_name, v.to_string().as_str(), ctx);
                match transform_result {
                    Ok(r) => match r {
                        Some(v) => {
                            if field.quote {
                                return Some(Value::from(v));
                            }
                            let tr_json = serde_json::from_str(v.as_str());
                            match tr_json {
                                Ok(json) => Some(json),
                                Err(e) => {
                                    err = Some(e.into());
                                    None
                                }
                            }
                        }
                        None => None,
                    },
                    Err(e) => {
                        err = Some(e.into());
                        None
                    }
                }
            });

            if let Some(e) = err {
                return Err(e);
            }

            match replace_result {
                Ok(v) => value = v,
                Err(e) => {
                    return Err(e.into());
                }
            }
        }
        Ok(value.to_string())
    }
}

impl Transformer for JsonTransformer {
    fn transform(
        &self,
        field_name: &str,
        field_value: &str,
        ctx: &Option<TransformContext>,
    ) -> TransformResult {
        match serde_json::from_str(field_value) {
            Ok(parsed_value) => match self.transform_parsed_value(field_name, parsed_value, ctx) {
                Ok(v) => TransformResult::present(v),
                Err(e) => TransformResult::error(field_name, field_value, e.to_string().as_str()),
            },
            Err(e) => {
                // invalid JSON from DB
                match &self.on_invalid {
                    OnInvalid::AsIs => TransformResult::present(field_value.to_string()),
                    OnInvalid::Error => {
                        TransformResult::error(field_name, field_value, e.to_string().as_str())
                    }
                    OnInvalid::ReplaceWith(replacement) => match replacement {
                        ReplaceInvalid::Plain(str) => TransformResult::present(str.clone()),
                        ReplaceInvalid::Rule(t) => t.transform(field_name, field_value, ctx),
                    },
                }
            }
        }
    }

    fn init(&mut self, ctx: &TransformerInitContext) {
        for field in &mut self.fields {
            field.rule.init(ctx)
        }
        if let OnInvalid::ReplaceWith(ReplaceInvalid::Rule(t)) = &mut self.on_invalid {
            t.init(ctx);
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq, Hash)]
struct Field {
    name: String,
    selector: Selector,
    #[serde(with = "serde_yaml::with::singleton_map")]
    rule: Transformers,
    #[serde(default)]
    quote: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq, Hash)]
#[serde(rename_all = "snake_case")]
enum OnInvalid {
    AsIs,
    ReplaceWith(ReplaceInvalid),
    Error,
}

impl Default for OnInvalid {
    fn default() -> Self {
        Self::ReplaceWith(ReplaceInvalid::default())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq, Hash)]
#[serde(untagged)]
enum ReplaceInvalid {
    Rule(Box<Transformers>),
    Plain(String),
}

impl Default for ReplaceInvalid {
    fn default() -> Self {
        Self::Plain("{}".to_string())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{utils::EnumWrapper, Transformers};
    use serde_json::json;

    #[test]
    fn transform() {
        let config = r#"
            json:
              fields:
                - name: "user_name"
                  selector: "$..user.name"
                  quote: true
                  rule:
                    template:
                      format: "UserName"
                - name: "user_age"
                  selector: "$..user.age"
                  rule:
                    random_num:
                      min: 25
                      max: 55
        "#;
        let json = json!(
            [
                { "user": { "name": "Andrew", "age": 20, "comment": "Abc" } },
                { "user": { "name": "Briana", "age": 20, "comment": "Def" } },
                { "user": { "name": "Charlie", "age": 20, "comment": "Ghi" } }
            ]
        );
        let mut t: Transformers = EnumWrapper::parse(config).unwrap();
        t.init(&TransformerInitContext::default());

        let new_json: Value = serde_json::from_str(
            t.transform("field", json.to_string().as_str(), &None)
                .unwrap()
                .unwrap()
                .as_str(),
        )
        .unwrap();
        for i in 0..=2 {
            let new_user = &new_json[i]["user"];
            assert_eq!(new_user["name"], "UserName");
            let age = new_user["age"].as_u64().unwrap();
            assert!(age >= 25 && age <= 55);
            assert_eq!(new_user["comment"], json[i]["user"]["comment"]);
        }
    }

    mod on_invalid {
        use super::*;

        #[test]
        fn default() {
            let config = r#"
                json:
                  fields:
                    - name: "user_name"
                      selector: "$..user.name"
                      quote: true
                      rule:
                        first_name: {}
                "#;

            let t: Transformers = EnumWrapper::parse(config).unwrap();
            let new_json = t.transform("field", "invalid", &None).unwrap().unwrap();

            assert_eq!(new_json, "{}");
        }

        #[test]
        fn as_is() {
            let config = r#"
                json:
                  fields:
                    - name: "user_name"
                      selector: "$..user.name"
                      quote: true
                      rule:
                        first_name: {}
                  on_invalid: as_is
                "#;

            let t: Transformers = EnumWrapper::parse(config).unwrap();
            let new_json = t.transform("field", "invalid", &None).unwrap().unwrap();

            assert_eq!(new_json, "invalid");
        }

        #[test]
        fn replace_with_plain() {
            let config = r#"
                json:
                  fields:
                    - name: "user_name"
                      selector: "$..user.name"
                      quote: true
                      rule:
                        first_name: {}
                  on_invalid:
                    replace_with: '{"plain": true}'
                "#;

            let t: Transformers = EnumWrapper::parse(config).unwrap();
            let new_json = t.transform("field", "invalid", &None).unwrap().unwrap();

            assert_eq!(new_json, "{\"plain\": true}");
        }

        #[test]
        fn replace_with_rule() {
            let config = r#"
                json:
                  fields:
                    - name: "user_name"
                      selector: "$..user.name"
                      quote: true
                      rule:
                        first_name: {}
                  on_invalid:
                    replace_with:
                      template:
                        format: '{"rule": true}'
                "#;

            let mut t: Transformers = EnumWrapper::parse(config).unwrap();
            t.init(&TransformerInitContext::default());
            let new_json = t.transform("field", "invalid", &None).unwrap().unwrap();

            assert_eq!(new_json, "{\"rule\": true}");
        }

        #[test]
        fn error() {
            let config = r#"
                json:
                  fields:
                    - name: "user_name"
                      selector: "$..user.name"
                      quote: true
                      rule:
                        first_name: {}
                  on_invalid: error
                "#;

            let t: Transformers = EnumWrapper::parse(config).unwrap();
            let result = t.transform("field", "invalid", &None);

            assert!(result.is_err());
        }
    }
}
