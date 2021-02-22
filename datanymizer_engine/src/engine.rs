use crate::errors::UnknownColumnError;
use crate::transformer::TransformContext;
use crate::{errors::EngineError, Settings, Transformer};
use std::borrow::Cow;
use std::collections::HashMap;

pub struct Engine {
    pub settings: Settings,
}

impl Engine {
    pub fn new(settings: Settings) -> Self {
        Self { settings }
    }

    pub fn process_row<'a>(
        &self,
        table: String,
        column_indexes: &HashMap<String, usize>,
        values: &'a Vec<&str>,
    ) -> Result<Vec<Cow<'a, str>>, EngineError> {
        let ts = self.settings.transformers_for(&table);

        let mut transformed_values = Vec::with_capacity(values.len());
        for &v in values {
            transformed_values.push(Cow::from(v));
        }

        for (field, tr) in ts {
            if let Some(&i) = column_indexes.get(field) {
                match tr.transform(
                    &format!("{}.{}", table, field),
                    values[i],
                    &Some(TransformContext::new(
                        &self.settings.globals,
                        Some(column_indexes),
                        Some(&transformed_values),
                    )),
                ) {
                    Ok(Some(res)) => {
                        transformed_values[i] = Cow::Owned(res);
                    }
                    Err(e) => return Err(EngineError::TransformFieldError(e)),
                    _ => {}
                }
            } else {
                return Err(EngineError::UnknownColumnError(UnknownColumnError {
                    field_name: field.clone(),
                }));
            }
        }

        Ok(transformed_values)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn process_row() {
        let config = r#"
          source: {}
          tables:
            - name: actor
              rules:
                first_name:
                  first_name: {}
                last_name:
                  last_name: {}
                last_update:
                  datetime:
                    from: 1990-01-01T00:00:00+00:00
                    to: 2010-12-31T00:00:00+00:00
        "#;
        let settings = Settings::from_yaml(config, String::new()).unwrap();

        let table = String::from("actor");
        let values = vec!["", "", "", "", ""];

        let mut column_indexes = HashMap::new();
        column_indexes.insert(String::from("first_name"), 0);
        column_indexes.insert(String::from("last_name"), 1);
        column_indexes.insert(String::from("last_update"), 4);

        let tr_values = Engine::new(settings)
            .process_row(table, &column_indexes, &values)
            .unwrap();

        assert_ne!(tr_values[0], "");
        assert_ne!(tr_values[1], "");
        assert_eq!(tr_values[2], "");
        assert_eq!(tr_values[3], "");
        assert_ne!(tr_values[4], "");
    }

    #[test]
    fn final_row() {
        let config = r#"
          source: {}
          tables:
            - name: some_table
              rule_order:
                - greeting
                - options
              rules:
                first_name:
                  first_name: {}
                last_name:
                  last_name: {}
                greeting:
                  template:
                    format: "Hello, {{ final.first_name }} {{ final.last_name }}!"
                options:
                  template:
                    format: "{greeting: \"{{ final.greeting }}\"}"
        "#;
        let settings = Settings::from_yaml(config, String::new()).unwrap();

        let table = String::from("some_table");
        let values = vec!["", "", "", "", ""];

        let mut column_indexes = HashMap::new();
        column_indexes.insert(String::from("first_name"), 0);
        column_indexes.insert(String::from("last_name"), 1);
        column_indexes.insert(String::from("greeting"), 3);
        column_indexes.insert(String::from("options"), 4);

        let tr_values = Engine::new(settings)
            .process_row(table, &column_indexes, &values)
            .unwrap();

        assert_ne!(tr_values[0], "");
        assert_ne!(tr_values[1], "");
        assert_eq!(tr_values[2], "");
        assert_eq!(
            tr_values[3],
            format!("Hello, {} {}!", tr_values[0], tr_values[1])
        );
        assert_eq!(tr_values[4], format!("{{greeting: \"{}\"}}", tr_values[3]));
    }

    #[test]
    fn pipeline_and_final_row() {
        let config = r#"
          source: {}
          tables:
            - name: some_table
              rule_order:
                - greeting
                - options
              rules:
                first_name:
                  first_name: {}
                last_name:
                  last_name: {}
                greeting:
                  pipeline:
                    pipes:
                      - template:
                          format: "dear {{ final.first_name }} {{ final.last_name }}"
                      - capitalize: ~
                options:
                  template:
                    format: "{greeting: \"{{ final.greeting }}\"}"
        "#;
        let settings = Settings::from_yaml(config, String::new()).unwrap();

        let table = String::from("some_table");
        let values = vec!["", "", "", "", ""];

        let mut column_indexes = HashMap::new();
        column_indexes.insert(String::from("first_name"), 0);
        column_indexes.insert(String::from("last_name"), 1);
        column_indexes.insert(String::from("greeting"), 3);
        column_indexes.insert(String::from("options"), 4);

        let tr_values = Engine::new(settings)
            .process_row(table, &column_indexes, &values)
            .unwrap();

        assert_ne!(tr_values[0], "");
        assert_ne!(tr_values[1], "");
        assert_eq!(tr_values[2], "");
        assert_eq!(
            tr_values[3],
            format!("Dear {} {}", tr_values[0], tr_values[1])
        );
        assert_eq!(tr_values[4], format!("{{greeting: \"{}\"}}", tr_values[3]));
    }
}
