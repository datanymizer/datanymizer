mod hash_functions;
mod store_functions;
mod time_functions;

use crate::{
    transformer::{
        TransformContext, TransformResult, TransformResultHelper, Transformer,
        TransformerInitContext,
    },
    Transformers,
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
///       format: "Hello, {{name}}! {{_1}}:{{_0 | upper}}"
///       rules:
///         - email:
///             kind: Safe
///       variables:
///         name: Alex
/// ```
///
/// where:
/// * `_0` - original value;
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
        let renderer = Tera::default();
        // renderer.add_raw_template(TEMPLATE_NAME, &format).unwrap();

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

    fn init(&mut self, ctx: &TransformerInitContext) {
        store_functions::register(&mut self.renderer, ctx.template_store.clone());
        hash_functions::register(&mut self.renderer);
        time_functions::register(&mut self.renderer);

        let mut ext_renderer = Tera::default();

        if let Some(templates) = &ctx.template_collection.raw {
            for (name, body) in templates {
                ext_renderer.add_raw_template(name, body).unwrap();
            }
        }

        if let Some(files) = &ctx.template_collection.files {
            for file in files.iter() {
                ext_renderer.add_template_file(&file, None).unwrap();
            }
        }

        self.renderer.extend(&ext_renderer).unwrap();

        if let Some(ts) = &mut self.rules {
            for t in ts {
                t.init(ctx);
            }
        }

        self.renderer
            .add_raw_template(TEMPLATE_NAME, &self.format)
            .unwrap();
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
        transformer::{Globals, TransformerDefaults},
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

        let mut transformer: Transformers = serde_yaml::from_str(config).unwrap();
        transformer.init(&TransformerInitContext::default());

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
    fn init() {
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
        t.init(&TransformerInitContext::from_defaults(
            TransformerDefaults {
                locale: LocaleConfig::RU,
            },
        ));

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
                let mut t: Transformers = serde_yaml::from_str(config).unwrap();
                t.init(&TransformerInitContext::default());

                return t;
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
                let mut t: Transformers = serde_yaml::from_str(config).unwrap();
                t.init(&TransformerInitContext::default());

                return t;
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
                let mut t: Transformers = serde_yaml::from_str(config).unwrap();
                t.init(&TransformerInitContext::default());

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
                let mut t: Transformers = serde_yaml::from_str(config).unwrap();
                t.init(&TransformerInitContext::default());

                return t;
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

    mod store {
        use super::*;

        fn read_transformer() -> Transformers {
            let config = r#"
                                template:
                                  format: "Read: {{ store_read(key=_0) }}"
                              "#;
            serde_yaml::from_str(config).unwrap()
        }

        fn read_default_transformer() -> Transformers {
            let config = r#"
                                template:
                                  format: "Read: {{ store_read(key=_0, default='def key') }}"
                              "#;
            serde_yaml::from_str(config).unwrap()
        }

        fn write_transformer() -> Transformers {
            let config = r#"
                  template:
                    format: "{{ store_force_write(key=_1, value=_2) }}Write: {{ _2 }} into {{ _1 }}"
                    rules:
                      - template:
                          format: '{{ "key_" ~ _0 }}'
                      - template:
                          format: '{{ "value_" ~ _0 }}'
                "#;
            serde_yaml::from_str(config).unwrap()
        }

        #[test]
        fn basic() {
            let mut r = read_transformer();
            let mut rd = read_default_transformer();
            let mut w = write_transformer();
            let ctx = TransformerInitContext::default();

            r.init(&ctx);
            rd.init(&ctx);
            w.init(&ctx);

            let value = w.transform("field", "a", &None).unwrap().unwrap();
            assert_eq!(value, "Write: value_a into key_a");

            let value = w.transform("field", "b", &None).unwrap().unwrap();
            assert_eq!(value, "Write: value_b into key_b");

            let value = w.transform("field", "c", &None).unwrap().unwrap();
            assert_eq!(value, "Write: value_c into key_c");

            let value = r.transform("field", "key_a", &None).unwrap().unwrap();
            assert_eq!(value, "Read: value_a");

            let value = r.transform("field", "key_b", &None).unwrap().unwrap();
            assert_eq!(value, "Read: value_b");

            let value = r.transform("field", "key_c", &None).unwrap().unwrap();
            assert_eq!(value, "Read: value_c");

            let value = rd.transform("field", "key_a", &None).unwrap().unwrap();
            assert_eq!(value, "Read: value_a");

            let value = rd.transform("field", "key_d", &None).unwrap().unwrap();
            assert_eq!(value, "Read: def key");
        }

        #[test]
        fn repeatable_read() {
            let mut r = read_transformer();
            let mut rd = read_default_transformer();
            let mut w = write_transformer();
            let ctx = TransformerInitContext::default();

            r.init(&ctx);
            rd.init(&ctx);
            w.init(&ctx);

            let value = w.transform("field", "a", &None).unwrap().unwrap();
            assert_eq!(value, "Write: value_a into key_a");

            let value = r.transform("field", "key_a", &None).unwrap().unwrap();
            assert_eq!(value, "Read: value_a");

            let value = r.transform("field", "key_a", &None).unwrap().unwrap();
            assert_eq!(value, "Read: value_a");

            let value = rd.transform("field", "key_d", &None).unwrap().unwrap();
            assert_eq!(value, "Read: def key");

            let value = rd.transform("field", "key_d", &None).unwrap().unwrap();
            assert_eq!(value, "Read: def key");
        }

        #[test]
        fn condition() {
            let config = r#"
                                template:
                                  format: |
                                    {%- set c = store_read(key=_0, default=false) -%}
                                    {%- if c -%}
                                      {{ c }}
                                    {%- endif -%}
                              "#;
            let mut t: Transformers = serde_yaml::from_str(config).unwrap();
            let mut w = write_transformer();
            let ctx = TransformerInitContext::default();

            t.init(&ctx);
            w.init(&ctx);

            let value = w.transform("field", "a", &None).unwrap().unwrap();
            assert_eq!(value, "Write: value_a into key_a");

            let value = t.transform("field", "key_a", &None).unwrap().unwrap();
            assert_eq!(value, "value_a");

            let value = t.transform("field", "b", &None).unwrap().unwrap();
            assert_eq!(value, "");
        }

        #[test]
        fn overwrite() {
            let config = r#"
                  template:
                    format: "{{ store_force_write(key=_1, value=_2) }}Write: {{ _2 }} into {{ _1 }}"
                    rules:
                      - template:
                          format: 'key'
                      - template:
                          format: '{{ "value_" ~ _0 }}'
                "#;

            let mut r = read_transformer();
            let mut w: Transformers = serde_yaml::from_str(config).unwrap();
            let ctx = TransformerInitContext::default();

            r.init(&ctx);
            w.init(&ctx);

            let value = w.transform("field", "a", &None).unwrap().unwrap();
            assert_eq!(value, "Write: value_a into key");

            let value = r.transform("field", "key", &None).unwrap().unwrap();
            assert_eq!(value, "Read: value_a");

            let value = w.transform("field", "b", &None).unwrap().unwrap();
            assert_eq!(value, "Write: value_b into key");

            let value = r.transform("field", "key", &None).unwrap().unwrap();
            assert_eq!(value, "Read: value_b");
        }

        #[test]
        fn inc() {
            let config = r#"
                  template:
                    format: '{{ store_inc(key="key", value=_0 | float) }}Write: {{ _0 | float }} into key'
                "#;

            let mut r = read_transformer();
            let mut w: Transformers = serde_yaml::from_str(config).unwrap();
            let ctx = TransformerInitContext::default();

            r.init(&ctx);
            w.init(&ctx);

            let value = w.transform("field", "0.5", &None).unwrap().unwrap();
            assert_eq!(value, "Write: 0.5 into key");

            let value = r.transform("field", "key", &None).unwrap().unwrap();
            assert_eq!(value, "Read: 0.5");

            let value = w.transform("field", "2", &None).unwrap().unwrap();
            assert_eq!(value, "Write: 2 into key");

            let value = r.transform("field", "key", &None).unwrap().unwrap();
            assert_eq!(value, "Read: 2.5");

            let value = w.transform("field", "-1", &None).unwrap().unwrap();
            assert_eq!(value, "Write: -1 into key");

            let value = r.transform("field", "key", &None).unwrap().unwrap();
            assert_eq!(value, "Read: 1.5");
        }
    }

    mod extended_templates {
        use crate::settings::TemplatesCollection;

        use super::*;

        #[test]
        fn using_macros() {
            let config = r#"
                template:
                    format: >
                      {% import "base" as macros -%}
                      {{ macros::decrement(n=10) }}"#;
            let macro_config = r#"
                  raw:
                    base: >
                      {% macro decrement(n) -%}
                      {% if n > 1 %}{{ n }}-{{ self::decrement(n=n-1) }}{% else %}1{% endif -%}
                      {% endmacro decrement -%}"#;
            let templates_collection: TemplatesCollection =
                serde_yaml::from_str(macro_config).unwrap();
            let mut t: Transformers = serde_yaml::from_str(config).unwrap();
            let mut context = TransformerInitContext::default();
            context.template_collection = templates_collection;
            t.init(&context);

            let value = t.transform("field", "", &None).unwrap().unwrap();
            assert_eq!(value, "10-9-8-7-6-5-4-3-2-1");
        }
    }

    // Test this because of using the fork
    mod tera_builtin_features {
        use time::OffsetDateTime;
        use super::*;

        fn transform_result(expr: &str) -> String {
            let config = format!(
                r#"
                        template:
                          format: '{{{{{}}}}}'
                      "#,
                expr
            );

            let mut transformer: Transformers = serde_yaml::from_str(config.as_str()).unwrap();
            transformer.init(&TransformerInitContext::default());

            transformer.transform("", "", &None).unwrap().unwrap()
        }

        fn assert_expected(expr: &str, expected: &str) {
            let res = transform_result(expr);
            assert_eq!(res, String::from(expected));
        }

        #[test]
        fn get_random() {
            assert_expected("get_random(start=123,end=124)", "123");
        }

        #[test]
        fn filesizeformat() {
            assert_expected("1024 | filesizeformat", "1 KB");
        }

        #[test]
        fn urlencode() {
            assert_expected("\"/path?a=b&c=d\" | urlencode", "/path%3Fa%3Db%26c%3Dd");
        }

        #[test]
        fn urlencode_strict() {
            assert_expected(
                "\"/path?a=b&c=d\" | urlencode_strict",
                "%2Fpath%3Fa%3Db%26c%3Dd",
            );
        }

        #[test]
        fn slugify() {
            assert_expected("\"Hello Everyone\" | slugify", "hello-everyone");
        }

        #[test]
        fn now() {
            let now = OffsetDateTime::now_utc();
            let res = transform_result("now()");
            assert!(res.starts_with(now.year().to_string().as_str()));
        }
    }
}
