use super::Globals;
use std::collections::HashMap;

#[derive(Clone)]
pub struct TransformContext<'a> {
    pub globals: &'a Option<Globals>,
    pub final_row: Option<&'a HashMap<String, String>>,
}

impl<'a> TransformContext<'a> {
    pub fn new(
        globals: &'a Option<Globals>,
        final_row: Option<&'a HashMap<String, String>>,
    ) -> Self {
        Self { globals, final_row }
    }
}

impl Default for TransformContext<'_> {
    fn default() -> Self {
        Self {
            globals: &None,
            final_row: None,
        }
    }
}
