use crate::{errors::EngineError, value::StringValue, Settings, Transformer};

pub struct Engine {
    pub settings: Settings,
}

impl Engine {
    pub fn new(settings: Settings) -> Self {
        Self { settings }
    }

    pub fn process(&self, value: &mut StringValue) -> Result<(), EngineError> {
        if let Some(tr) = self
            .settings
            .lookup_transformers(value.table_name.clone(), value.field_name.clone())
        {
            match tr.transform(
                &value.field_name,
                value.value.clone().as_ref(),
                &self.settings.globals,
            ) {
                Ok(Some(res)) => value.update(res),
                Err(e) => return Err(EngineError::TransformFieldError(e)),
                _ => {}
            }
        }
        Ok(())
    }
}
