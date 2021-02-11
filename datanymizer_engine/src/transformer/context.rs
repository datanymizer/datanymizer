use crate::Globals;

pub struct TransformContext<'a> {
    pub globals: &'a Option<Globals>,
    // pub row: Option<&'a HashMap<String, String>>,
}

impl<'a> TransformContext<'a> {
    pub fn new(globals: &'a Option<Globals>) -> Self {
        Self { globals }
    }
}

impl Default for TransformContext<'_> {
    fn default() -> Self {
        Self { globals: &None }
    }
}
