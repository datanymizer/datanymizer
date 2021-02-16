use crate::{
    transformer::{Globals, TransformResult, TransformResultHelper, Transformer},
    TransformerDefaults, Transformers,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use tera::{Context, Tera};

const TEMPLATE_NAME: &str = "TemplateTransformerTemplate";

/// Using a templating engine to generate or transform values.
/// [Tera](https://tera.netlify.app/) is used as a template engine in this transformer.
///
/// # Example:
///
/// ```yaml
/// #...
/// rules:
///   field_name:
///     template:
///       format: "Hello, {{name}}! {{_1}}:{{_0 | upper}}
///       rules:
///         - email:
///             kind: Safe
///       variables:
///         name: Alex
/// ```
///
/// where:
/// * `_0` - transformed value;
/// * `_1` and `_N` - Rules by index (started from `1`). You can use any transformer from engine;
/// * `{{name}}` - Named variable from `variables` config;
///
/// Also, you can use any filter or markup from [Tera](tera.netlify.app/) template engine.
#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone)]
pub struct TemplateTransformer {
    pub format: String,
    pub rules: Option<Vec<Transformers>>,
    pub variables: Option<HashMap<String, Value>>,
}

impl Default for TemplateTransformer {
    fn default() -> Self {
        Self {
            format: String::new(),
            rules: None,
            variables: None,
        }
    }
}

#[allow(clippy::derive_hash_xor_eq)]
impl Hash for TemplateTransformer {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.format.hash(state);
        if let Some(rules) = self.rules.clone() {
            for rule in rules {
                rule.hash(state);
            }
        }
        if let Some(variables) = self.variables.clone() {
            for (k, _) in variables {
                k.hash(state);
            }
        }
    }
}

impl Transformer for TemplateTransformer {
    fn transform(
        &self,
        field_name: &str,
        field_value: &str,
        globals: &Option<Globals>,
    ) -> TransformResult {
        let mut rules_names: HashMap<String, Value> = HashMap::new();
        if let Some(rules) = self.rules.clone() {
            for (i, rule) in rules.iter().enumerate() {
                let key = format!("_{}", i + 1);
                let transform_result: Option<String> =
                    rule.transform(field_name, field_value, globals)?;
                let value = transform_result.unwrap_or_else(|| "".to_string());
                rules_names.insert(key, Value::String(value));
            }
        }

        let mut vars = self.variables.clone().unwrap_or_default();

        if let Some(items) = globals {
            vars.extend(items.clone());
        }

        vars.extend(rules_names);
        vars.insert("_0".to_string(), Value::String(field_value.to_string()));

        let mut tera = Tera::default();
        tera.add_raw_template(TEMPLATE_NAME, &self.format)?;
        let mut context = Context::new();
        for (k, v) in vars {
            context.insert(k, &v);
        }

        match tera.render(TEMPLATE_NAME, &context) {
            Ok(res) => TransformResult::present(res),
            Err(e) => TransformResult::error(field_name, field_value, &e.to_string()),
        }
    }

    fn set_defaults(&mut self, defaults: &TransformerDefaults) {
        if let Some(ts) = &mut self.rules {
            for t in ts {
                t.set_defaults(defaults);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        transformer::Globals,
        transformers::{CityTransformer, NoneTransformer, PersonNameTransformer},
        LocaleConfig, Transformers,
    };
    use serde_json::Value;

    #[test]
    fn template_interpolation() {
        let expected: String = String::from("Hello, ALEX! Any text with replace:Dr, global: test");
        let mut global_values = Globals::new();
        global_values.insert(
            "global_value_1".to_string(),
            Value::String("test".to_string()),
        );

        let config = r#"
template:
    format: "Hello, {{name | upper }}! {{_1}} with replace:{{_0 | replace(from=\"Mr\", to=\"Dr\")}}, global: {{ global_value_1 }}"
    rules:
      - template:
            format: Any text
    variables:
        name: Alex
"#;

        let transformer: Transformers = serde_yaml::from_str(config).unwrap();

        let res = transformer.transform("", "Mr", &Some(global_values));
        assert_eq!(res, Ok(Some(expected)));
    }

    #[test]
    fn set_defaults() {
        let mut t = TemplateTransformer {
            format: String::new(),
            rules: Some(vec![
                Transformers::City(CityTransformer::default()),
                Transformers::PersonName(PersonNameTransformer {
                    locale: Some(LocaleConfig::ZH_TW),
                }),
                Transformers::None(NoneTransformer),
            ]),
            variables: None,
        };
        t.set_defaults(&TransformerDefaults {
            locale: LocaleConfig::RU,
        });

        let rules = t.rules.unwrap();

        assert!(matches!(&rules[0], Transformers::City(t) if t.locale == Some(LocaleConfig::RU)));
        assert!(
            matches!(&rules[1], Transformers::PersonName(t) if t.locale == Some(LocaleConfig::ZH_TW))
        );
    }
}
