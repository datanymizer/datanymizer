use crate::errors::UnknownColumnError;
use crate::transformer::TransformContext;
use crate::{errors::EngineError, Settings, Transformer};
use std::collections::HashMap;

pub struct Engine {
    pub settings: Settings,
}

impl Engine {
    pub fn new(settings: Settings) -> Self {
        Self { settings }
    }

    pub fn process_row(
        &self,
        table: String,
        values: Vec<&str>,
        column_indexes: &HashMap<String, usize>,
    ) -> Result<Vec<String>, EngineError> {
        let with_ctx = true;
        let ts = self.settings.transformers_for(&table);

        let mut transformed_values: Vec<String> = values.iter().map(|v| v.to_string()).collect();

        if with_ctx {
            let mut transformed_value_map = HashMap::with_capacity(ts.len());

            for (field, tr) in ts {
                if let Some(&i) = column_indexes.get(field) {
                    match tr.transform(
                        &format!("{}.{}", table, field),
                        values[i],
                        Some(TransformContext::new(Some(&transformed_value_map))),
                    ) {
                        Ok(Some(res)) => {
                            transformed_value_map.insert(field.to_string(), res.clone());
                            transformed_values[i] = res;
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
        } else {
            for (field, tr) in ts {
                if let Some(&i) = column_indexes.get(field) {
                    match tr.transform(&format!("{}.{}", table, field), values[i], None) {
                        Ok(Some(res)) => {
                            transformed_values[i] = res;
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
        }

        Ok(transformed_values)
    }
}
