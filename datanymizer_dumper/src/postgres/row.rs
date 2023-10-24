use super::escaper;
use crate::Table;
use anyhow::Result;
use datanymizer_engine::Engine;
use postgres::types::Type;
use std::{borrow::Cow, char};

#[derive(Debug)]
pub struct PgRow<T>
where
    T: Table<Type>,
{
    table: T,
    source: String,
}

impl<T> PgRow<T>
where
    T: Table<Type>,
{
    pub fn from_string_row(source: String, parent_table: T) -> Self {
        Self {
            source,
            table: parent_table,
        }
    }

    /// Applies the transform engine to every column in the row
    /// Returns a new StringRecord for store in the dump
    pub fn transform(&self, engine: &Engine, cfg_tbl_name: &str) -> Result<String> {
        let split_char: char = char::from_u32(0x0009).unwrap();
        let values: Vec<_> = self.source.split(split_char).collect();
        let mut transformed_values = engine.process_row(
            String::from(cfg_tbl_name),
            self.table.get_column_indexes(),
            &values,
        )?;
        for v in &mut transformed_values {
            if let Cow::Owned(ref mut s) = v {
                escaper::replace_chars(s);
            }
        }

        Ok(transformed_values.join("\t"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::postgres::{column::PgColumn, table::PgTable};
    use datanymizer_engine::Settings;

    fn new_column(position: i32, name: &str) -> PgColumn {
        PgColumn {
            position,
            name: String::from(name),
            data_type: String::new(),
            inner_type: Some(0),
        }
    }

    #[test]
    fn transform() {
        let config = r#"
          source: {}
          tables:
            - name: table_name
              rules:
                first_name:
                  capitalize: ~
                middle_name:
                  capitalize: ~
                last_name:
                  capitalize: ~
                comment:
                  template:
                    format: |
                      Multi
                      line
        "#;
        let settings = Settings::from_yaml(config).unwrap();

        let mut table = PgTable::new("table_name".to_string(), "public".to_string());

        let col1 = new_column(1, "first_name");
        let col2 = new_column(2, "middle_name");
        let col3 = new_column(3, "last_name");
        let col4 = new_column(4, "comment");

        table.set_columns(vec![col1, col2, col3, col4]);
        let row = PgRow::from_string_row("first\tmiddle\tlast\t".to_string(), table);

        assert_eq!(
            row.transform(&Engine::new(settings), "table_name").unwrap(),
            "First\tMiddle\tLast\tMulti\\nline\\n"
        );
    }
}
