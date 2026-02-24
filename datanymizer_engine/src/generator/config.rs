use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub tables: Vec<Table>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Table {
    pub name: String,
    pub row_count: usize,
    pub key: Option<PrimaryKey>,
    #[serde(default)]
    pub foreign_keys: Vec<ForeignKey>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct PrimaryKey {
    pub name: String,
    #[serde(flatten)]
    pub options: HashMap<String, String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ForeignSource {
    pub name: String,
    pub table_name: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ForeignKey {
    pub name: String,
    #[serde(default)]
    pub kind: ForeignKeyKind,
    pub source: ForeignSource,

    #[serde(flatten)]
    pub options: HashMap<String, String>,
}

#[derive(Debug, Deserialize, Clone, Default)]
#[serde(rename_all = "snake_case")]
pub enum ForeignKeyKind {
    Monotonic,
    Random,
    #[default]
    MonotonicRandom,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse() {
        let config = r#"
          tables:
            - name: public.country
              row_count: 1
              key:
                name: country_id
            - name: public.city
              row_count: 100
              key:
                name: city_id
              foreign_keys:
                - name: country_id
                  source:
                    table_name: public.country
                    name: country_id
            - name: public.address
              row_count: 1000
              key:
                name: address_id
              foreign_keys:
                - name: city_id
                  source:
                    table_name: public.city
                    name: city_id
            "#;

        let c: Config = serde_yaml::from_str(config).unwrap();

        assert_eq!(c.tables.len(), 3);

        assert_eq!(c.tables[0].name, "public.country");
        assert_eq!(c.tables[0].row_count, 1);
        assert_eq!(c.tables[1].name, "public.city");
        assert_eq!(c.tables[1].row_count, 100);
        assert_eq!(c.tables[2].name, "public.address");
        assert_eq!(c.tables[2].row_count, 1000);
    }
}
