use config::ForeignKeyKind;
use foreign_key::{default_seq_to_rand, MonotonicFKey, MonotonicRandomFKey, RandomFKey};
use key::{Key, MonotonicKey};
use std::{collections::HashMap, sync::Arc};
use table::{GenTable, GenTableIter, KeyColMap};

mod config;
mod foreign_key;
mod key;
mod seq_to_rand;
mod table;

pub use config::Config as GeneratorConfig;

const DEFAULT_PRIMARY_KEY_FROM: usize = 1;

pub struct Generator {
    tables: HashMap<String, GenTable>,
}

impl Generator {
    pub fn contains_table(&self, name: &str) -> bool {
        self.tables.contains_key(name)
    }

    pub fn iter_for_table(
        &self,
        name: &str,
        column_indexes: HashMap<String, usize>,
    ) -> Option<GenTableIter> {
        self.tables
            .get(name)
            .map(|table| GenTableIter::new(table, column_indexes))
    }

    pub fn from_config(cfg: &GeneratorConfig) -> Self {
        let mut row_counts = HashMap::new();
        let mut all_keys: HashMap<String, KeyColMap> = HashMap::new();

        // process primary keys
        for table_cfg in &cfg.tables {
            let mut table_keys = KeyColMap::new();
            if let Some(key) = table_cfg.key.as_ref() {
                let key_name = key.name.clone();
                let from: usize = key
                    .options
                    .get("from")
                    .map(|s| s.parse().expect("Invalid from value"))
                    .unwrap_or(DEFAULT_PRIMARY_KEY_FROM);
                table_keys.insert(
                    key_name,
                    Arc::new(Box::new(MonotonicKey::new(from, table_cfg.row_count))),
                );
            }
            all_keys.insert(table_cfg.name.clone(), table_keys);
            row_counts.insert(table_cfg.name.clone(), table_cfg.row_count);
        }

        // process foreign keys
        let mut fkey_total = cfg
            .tables
            .iter()
            .fold(0, |acc, t| acc + t.foreign_keys.len());
        while fkey_total > 0 {
            let prev_count = fkey_total;
            for table_cfg in &cfg.tables {
                for key in table_cfg.foreign_keys.iter() {
                    let src_name = &key.source.name;
                    let src_table_name = &key.source.table_name;

                    let src_table_keys = &all_keys[src_table_name];
                    // if the source has been processed already
                    if let Some(src) = src_table_keys.get(src_name) {
                        let src = src.clone();
                        if let Some(table_keys) = all_keys.get_mut(&table_cfg.name) {
                            table_keys.insert(
                                key.name.clone(),
                                Self::get_foreign_key(&key.kind, table_cfg.row_count, src),
                            );

                            fkey_total -= 1;
                        }
                    }
                }
            }
            // when can't find some foreign key sources
            if prev_count == fkey_total {
                panic!("Generator keys config error")
            }
        }

        let mut tables = HashMap::new();
        for (name, keys) in all_keys {
            tables.insert(name.clone(), GenTable::new(row_counts[&name], keys));
        }

        Self { tables }
    }

    fn get_foreign_key(
        kind: &ForeignKeyKind,
        row_count: usize,
        src: Arc<Box<dyn Key>>,
    ) -> Arc<Box<dyn Key>> {
        match kind {
            ForeignKeyKind::Monotonic => Arc::new(Box::new(MonotonicFKey::new(src, row_count))),
            ForeignKeyKind::Random => Arc::new(Box::new(RandomFKey::new(
                src,
                row_count,
                default_seq_to_rand(),
            ))),
            ForeignKeyKind::MonotonicRandom => Arc::new(Box::new(MonotonicRandomFKey::new(
                src,
                row_count,
                default_seq_to_rand(),
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{config::Config, *};

    #[test]
    fn from_config() {
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

        let cfg: Config = serde_yaml::from_str(config).unwrap();
        let g = Generator::from_config(&cfg);

        assert_eq!(g.tables.len(), 3);
        assert!(g.contains_table("public.country"));
        assert!(g.contains_table("public.city"));
        assert!(g.contains_table("public.address"));
    }
}
