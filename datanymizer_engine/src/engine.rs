use crate::{value::StringValue, Settings, Transformer};
use anyhow::Result;

pub struct Engine {
    pub settings: Settings,
}

impl Engine {
    pub fn new(settings: Settings) -> Self {
        Self { settings }
    }

    pub fn process(&self, value: &mut StringValue) -> Result<()> {
        if let Some(tr) = self
            .settings
            .lookup_transformers(value.table_name.clone(), value.field_name.clone())
        {
            if let Ok(Some(res)) = tr.transform(
                &value.field_name,
                value.value.clone().as_ref(),
                &self.settings.globals,
            ) {
                value.update(res);
            }
        }
        Ok(())
    }
}
