use super::Globals;
use std::{borrow::Cow, collections::HashMap};

#[derive(Clone)]
pub struct TransformContext<'a> {
    pub globals: &'a Option<Globals>,
    column_indexes: Option<&'a HashMap<String, usize>>,
    prev_row: Option<&'a [&'a str]>,
    final_row: Option<&'a Vec<Cow<'a, str>>>,
}

impl<'a> TransformContext<'a> {
    pub fn new(
        globals: &'a Option<Globals>,
        column_indexes: Option<&'a HashMap<String, usize>>,
        prev_row: Option<&'a [&'a str]>,
        final_row: Option<&'a Vec<Cow<'a, str>>>,
    ) -> Self {
        Self {
            globals,
            column_indexes,
            prev_row,
            final_row,
        }
    }

    pub fn prev_row_map(&self) -> Option<HashMap<&String, &str>> {
        if let Some(row) = self.prev_row {
            if let Some(column_indexes) = self.column_indexes {
                let mut row_map = HashMap::with_capacity(row.len());
                for (k, &i) in column_indexes.iter() {
                    row_map.insert(k, row[i]);
                }

                return Some(row_map);
            }
        }

        None
    }

    pub fn final_row_map(&self) -> Option<HashMap<&String, &String>> {
        if let Some(row) = self.final_row {
            if let Some(column_indexes) = self.column_indexes {
                let mut row_map = HashMap::with_capacity(row.len());
                for (k, &i) in column_indexes.iter() {
                    if let Cow::Owned(ref already_transformed) = row[i] {
                        row_map.insert(k, already_transformed);
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
            prev_row: None,
            final_row: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn row_maps() {
        let mut column_indexes = HashMap::new();
        column_indexes.insert(String::from("first_name"), 0);
        column_indexes.insert(String::from("middle_name"), 1);
        column_indexes.insert(String::from("last_name"), 2);
        column_indexes.insert(String::from("options"), 3);

        let prev_row = vec!["First", "Middle", "Last", "{}"];
        // first_name and last_name are already transformed
        let final_row = vec![
            Cow::Owned("t_First".to_string()),
            Cow::Borrowed(prev_row[1]),
            Cow::Owned("t_Last".to_string()),
            Cow::Borrowed(prev_row[3]),
        ];

        let ctx = TransformContext::new(
            &None,
            Some(&column_indexes),
            Some(&prev_row),
            Some(&final_row),
        );

        let prev_row_map = ctx.prev_row_map().unwrap();
        assert_eq!(prev_row_map.len(), 4);
        assert_eq!(prev_row_map[&"first_name".to_string()], "First");
        assert_eq!(prev_row_map[&"middle_name".to_string()], "Middle");
        assert_eq!(prev_row_map[&"last_name".to_string()], "Last");
        assert_eq!(prev_row_map[&"options".to_string()], "{}");

        let final_row_map = ctx.final_row_map().unwrap();
        assert_eq!(final_row_map.len(), 2);
        assert_eq!(final_row_map[&"first_name".to_string()], "t_First");
        assert_eq!(final_row_map[&"last_name".to_string()], "t_Last");
    }
}
