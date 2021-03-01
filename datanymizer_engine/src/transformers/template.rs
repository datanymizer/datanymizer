use crate::{
    transformer::{TransformContext, TransformResult, TransformResultHelper, Transformer},
    TransformerDefaults, Transformers,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
    collections::HashMap,
    hash::{Hash, Hasher},
};
use tera::{Context, Tera};

const TEMPLATE_NAME: &str = "TemplateTransformerTemplate";
const FINAL_ROW_KEY: &str = "final";
const PREV_ROW_KEY: &str = "prev";

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
#[derive(Serialize, Deserialize, Debug, Default, Clone)]
#[serde(from = "Config")]
pub struct TemplateTransformer {
    pub format: String,
    pub rules: Option<Vec<Transformers>>,
    pub variables: Option<HashMap<String, Value>>,

    #[serde(skip)]
    renderer: Tera,
}

impl TemplateTransformer {
    pub fn new(
        format: String,
        rules: Option<Vec<Transformers>>,
        variables: Option<HashMap<String, Value>>,
    ) -> Self {
        let mut renderer = Tera::default();
        renderer.add_raw_template(TEMPLATE_NAME, &format).unwrap();

        Self {
            format,
            rules,
            variables,
            renderer,
        }
    }

    fn render(&self, ctx: &Context) -> tera::Result<String> {
        self.renderer.render(TEMPLATE_NAME, ctx)
    }
}

impl PartialEq for TemplateTransformer {
    fn eq(&self, other: &Self) -> bool {
        self.format == other.format
            && self.rules == other.rules
            && self.variables == other.variables
    }
}

impl Eq for TemplateTransformer {}

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
        ctx: &Option<TransformContext>,
    ) -> TransformResult {
        let mut rules_names: HashMap<String, Value> = HashMap::new();
        if let Some(rules) = self.rules.clone() {
            for (i, rule) in rules.iter().enumerate() {
                let key = format!("_{}", i + 1);
                let transform_result: Option<String> =
                    rule.transform(field_name, field_value, ctx)?;
                let value = transform_result.unwrap_or_else(|| "".to_string());
                rules_names.insert(key, Value::String(value));
            }
        }

        let mut render_context = Context::new();
        let mut vars = self.variables.clone().unwrap_or_default();

        if let Some(c) = ctx {
            if let Some(items) = c.globals {
                vars.extend(items.clone());
            }

            if let Some(row_map) = c.final_row_map() {
                render_context.insert(FINAL_ROW_KEY, &row_map);
            }

            if let Some(row_map) = c.prev_row_map() {
                render_context.insert(PREV_ROW_KEY, &row_map);
            }
        }

        vars.extend(rules_names);
        vars.insert("_0".to_string(), Value::String(field_value.to_string()));

        for (k, v) in vars {
            render_context.insert(k, &v);
        }

        match self.render(&render_context) {
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

impl From<Config> for TemplateTransformer {
    fn from(cfg: Config) -> Self {
        Self::new(cfg.format, cfg.rules, cfg.variables)
    }
}

#[derive(Deserialize)]
struct Config {
    pub format: String,
    pub rules: Option<Vec<Transformers>>,
    pub variables: Option<HashMap<String, Value>>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transformer::TransformError;
    use crate::{
        transformer::Globals,
        transformers::{CityTransformer, NoneTransformer, PersonNameTransformer},
        LocaleConfig, Transformers,
    };
    use serde_json::Value;
    use std::borrow::Cow;

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
        let res = transformer.transform(
            "",
            "Mr",
            &Some(TransformContext::new(
                &Some(global_values),
                None,
                None,
                None,
            )),
        );
        assert_eq!(res, Ok(Some(expected)));
    }

    #[test]
    fn set_defaults() {
        let mut t = TemplateTransformer::new(
            String::new(),
            Some(vec![
                Transformers::City(CityTransformer::default()),
                Transformers::PersonName(PersonNameTransformer {
                    locale: Some(LocaleConfig::ZH_TW),
                }),
                Transformers::None(NoneTransformer),
            ]),
            None,
        );
        t.set_defaults(&TransformerDefaults {
            locale: LocaleConfig::RU,
        });

        let rules = t.rules.unwrap();

        assert!(matches!(&rules[0], Transformers::City(t) if t.locale == Some(LocaleConfig::RU)));
        assert!(
            matches!(&rules[1], Transformers::PersonName(t) if t.locale == Some(LocaleConfig::ZH_TW))
        );
    }

    mod row_refs {
        use super::*;

        fn column_indexes() -> HashMap<String, usize> {
            let mut column_indexes = HashMap::new();
            column_indexes.insert(String::from("first_name"), 0);
            column_indexes.insert(String::from("last_name"), 2);
            column_indexes
        }

        mod prev_row {
            use super::*;

            fn transformer() -> Transformers {
                let config = r#"
                                     template:
                                       format: "Hello, {{ prev.first_name }} {{ prev.last_name }}!"
                                  "#;
                serde_yaml::from_str(config).unwrap()
            }

            #[test]
            fn interpolation() {
                let expected: String = String::from("Hello, FIRST LAST!");

                let prev_row = vec!["FIRST", "MIDDLE", "LAST"];

                let res = transformer().transform(
                    "",
                    "Sensitive",
                    &Some(TransformContext::new(
                        &None,
                        Some(&column_indexes()),
                        Some(&prev_row),
                        None,
                    )),
                );

                assert_eq!(res, Ok(Some(expected)));
            }
        }

        mod final_row {
            use super::*;

            fn transformer() -> Transformers {
                let config = r#"
                                 template:
                                   format: "Hello, {{ final.first_name }} {{ final.last_name }}!"
                              "#;
                serde_yaml::from_str(config).unwrap()
            }

            #[test]
            fn interpolation() {
                let expected: String = String::from("Hello, FIRST LAST!");

                let final_row = vec![
                    Cow::Owned(String::from("FIRST")),
                    Cow::Borrowed("untransformed"),
                    Cow::Owned(String::from("LAST")),
                ];

                let res = transformer().transform(
                    "",
                    "Sensitive",
                    &Some(TransformContext::new(
                        &None,
                        Some(&column_indexes()),
                        None,
                        Some(&final_row),
                    )),
                );

                assert_eq!(res, Ok(Some(expected)));
            }

            #[test]
            fn nested_interpolation() {
                let expected: String = String::from("Hello, FIRST LAST!");

                let final_row = vec![
                    Cow::Owned(String::from("FIRST")),
                    Cow::Borrowed("untransformed"),
                    Cow::Owned(String::from("LAST")),
                ];

                let config = r#"
                                 template:
                                   format: "Hello, {{ final.first_name }} {{ _1 }}!"
                                   rules:
                                     - template:
                                         format: "{{ final.last_name }}"
                              "#;
                let t: Transformers = serde_yaml::from_str(config).unwrap();

                let res = t.transform(
                    "",
                    "Sensitive",
                    &Some(TransformContext::new(
                        &None,
                        Some(&column_indexes()),
                        None,
                        Some(&final_row),
                    )),
                );

                assert_eq!(res, Ok(Some(expected)));
            }

            #[test]
            fn ref_to_untransformed_value() {
                let final_row = vec![
                    Cow::Owned(String::from("FIRST")),
                    Cow::Borrowed("untransformed"),
                    Cow::Borrowed("untransformed"),
                ];

                let res = transformer().transform(
                    "",
                    "Sensitive",
                    &Some(TransformContext::new(
                        &None,
                        Some(&column_indexes()),
                        None,
                        Some(&final_row),
                    )),
                );

                assert_eq!(
                    res,
                    Err(TransformError {
                        field_name: String::from(""),
                        field_value: String::from("Sensitive"),
                        reason: String::from("Failed to render \'TemplateTransformerTemplate\'")
                    })
                );
            }
        }

        mod both_rows {
            use super::*;

            fn transformer() -> Transformers {
                let config = r#"
                                     template:
                                       format: "Hello, {{ prev.first_name }} {{ final.last_name }}!"
                                  "#;
                serde_yaml::from_str(config).unwrap()
            }

            #[test]
            fn interpolation() {
                let expected: String = String::from("Hello, FIRST tLAST!");

                let prev_row = vec!["FIRST", "MIDDLE", "LAST"];

                let final_row = vec![
                    Cow::Owned(String::from("tFIRST")),
                    Cow::Borrowed("untransformed"),
                    Cow::Owned(String::from("tLAST")),
                ];

                let res = transformer().transform(
                    "",
                    "Sensitive",
                    &Some(TransformContext::new(
                        &None,
                        Some(&column_indexes()),
                        Some(&prev_row),
                        Some(&final_row),
                    )),
                );

                assert_eq!(res, Ok(Some(expected)));
            }
        }
    }
}
