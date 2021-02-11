use crate::{errors::EngineError, Settings, Transformer};
use std::collections::HashMap;
use crate::transformer::TransformContext;

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
        values: &mut HashMap<String, String>,
    ) -> Result<(), EngineError> {
        let ctx = TransformContext::new(&self.settings.globals);
        for (field, tr) in self.settings.transformers_for(&table) {
            if let Some(value) = values.get_mut(field) {
                match tr.transform(
                    &format!("{}.{}", table, field),
                    value,
                    &ctx,
                ) {
                    Ok(Some(res)) => *value = res,
                    Err(e) => return Err(EngineError::TransformFieldError(e)),
                    _ => {}
                }
            }
        }
        Ok(())
    }
}
