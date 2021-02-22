use super::Globals;
use std::{borrow::Cow, collections::HashMap};

#[derive(Clone)]
pub struct TransformContext<'a> {
    pub globals: &'a Option<Globals>,
    column_indexes: Option<&'a HashMap<String, usize>>,
    final_row: Option<&'a Vec<Cow<'a, str>>>,
}

impl<'a> TransformContext<'a> {
    pub fn new(
        globals: &'a Option<Globals>,
        column_indexes: Option<&'a HashMap<String, usize>>,
        final_row: Option<&'a Vec<Cow<'a, str>>>,
    ) -> Self {
        Self {
            globals,
            column_indexes,
            final_row,
        }
    }

    pub fn final_row_map(&self) -> Option<HashMap<&String, &String>> {
        if let Some(row) = self.final_row {
            if let Some(column_indexes) = self.column_indexes {
                let mut row_map = HashMap::with_capacity(row.len());
                for (k, &i) in column_indexes.iter() {
                    match row[i] {
                        Cow::Owned(ref already_transformed) => {
                            row_map.insert(k, already_transformed);
                        }
                        _ => {}
                    }
                }

                return Some(row_map);
            }
        }

        None
    }
}

impl Default for TransformContext<'_> {
    fn default() -> Self {
        Self {
            globals: &None,
            column_indexes: None,
            final_row: None,
        }
    }
}
