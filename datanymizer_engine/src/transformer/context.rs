use std::collections::HashMap;

#[derive(Clone, Default)]
pub struct TransformContext<'a> {
    pub row: Option<&'a HashMap<String, String>>,
}

impl<'a> TransformContext<'a> {
    pub fn new(row: Option<&'a HashMap<String, String>>) -> Self {
        Self { row }
    }
}
