mod assert;
mod filter;
mod table;
mod templates;

use crate::{
    transformer::{TransformerDefaults, TransformerInitContext},
    transformers::Transformers,
    Transformer,
};
use anyhow::Result;
use config::{Config, ConfigError, File, FileFormat};
use serde::Deserialize;
use serde_json::Value as JsonValue;
use std::collections::HashMap;
use wildmatch::WildMatch;

pub use filter::{Filter, TableList};
pub use r#assert::{
    Assert, AssertError, AssertExpectation, AssertScope, AssertSeverity, ScalarExpectations,
    TableAssert,
};
pub use table::{Query, Table};
pub use templates::TemplatesCollection;

pub type Tables = Vec<Table>;

type TransformList = Vec<(String, Transformers)>;

#[derive(Debug, Deserialize, Clone)]
pub struct Settings {
    /// Tables list with transformation rules
    #[serde(default)]
    pub tables: Tables,

    /// Table order. All tables not listed are dumping at the beginning
    #[serde(default)]
    pub table_order: Vec<String>,

    /// Default transformers configuration
    #[serde(default)]
    pub default: TransformerDefaults,

    #[serde(default)]
    pub filter: Filter,

    /// SQL assertions executed before the dump starts.
    #[serde(default)]
    pub asserts: Vec<Assert>,

    /// Global values. Visible in any template.
    /// They may be shadowed by template variables.
    pub globals: Option<HashMap<String, JsonValue>>,

    pub templates: Option<TemplatesCollection>,

    #[serde(skip)]
    transform_map: Option<HashMap<String, TransformList>>,
}

impl Settings {
    pub fn new(path: String) -> Result<Self, ConfigError> {
        Self::from_source(File::with_name(&path))
    }

    pub fn from_yaml(config: &str) -> Result<Self, ConfigError> {
        Self::from_source(File::from_str(config, FileFormat::Yaml))
    }

    fn from_source<S>(source: S) -> Result<Self, ConfigError>
    where
        S: config::Source + Send + Sync + 'static,
    {
        let c = Config::builder().add_source(source).build()?;

        let mut settings: Self = c.try_deserialize()?;
        settings.validate()?;
        settings.preprocess();

        Ok(settings)
    }

    fn validate(&self) -> Result<(), ConfigError> {
        for (i, table) in self.tables.iter().enumerate() {
            let has_name = !table.name.is_empty();
            let has_names = table.names.as_ref().is_some_and(|n| !n.is_empty());

            if has_name && has_names {
                return Err(ConfigError::Message(format!(
                    "tables[{}]: cannot specify both `name` and `names` — use one or the other",
                    i
                )));
            }
            if !has_name && !has_names {
                return Err(ConfigError::Message(format!(
                    "tables[{}]: must specify either `name` or `names`",
                    i
                )));
            }
        }
        Ok(())
    }

    pub fn transformers_for(&self, table: &str) -> Option<&TransformList> {
        if let Some(m) = &self.transform_map {
            m.get(table)
        } else {
            panic!("No transform map");
        }
    }

    pub fn get_table(&self, name: &str) -> Option<&Table> {
        self.tables.iter().find(|t| t.name == name)
    }

    /// For a table that has been registered via [`register_table_transforms`], returns:
    /// - the display label of the config rule that matched it (e.g. `public.*`)
    /// - the list of anonymized column names
    ///
    /// Returns `None` if no config rule matches or no columns are anonymized.
    pub fn dry_run_info<T: AsRef<str>>(
        &self,
        full_name: &str,
        names: &[T],
    ) -> Option<(String, Vec<String>)> {
        let table_cfg = self.find_table(names)?;
        let transform_key = if table_cfg.has_wildcards() || table_cfg.name.is_empty() {
            full_name.to_string()
        } else {
            table_cfg.name.clone()
        };

        let transforms = self.transformers_for(&transform_key)?;
        if transforms.is_empty() {
            return None;
        }

        let rule_label = table_cfg
            .patterns()
            .into_iter()
            .find(|pat| names.iter().any(|n| WildMatch::new(pat).matches(n.as_ref())))
            .map(|s| s.to_string())
            .unwrap_or_else(|| table_cfg.name.clone());

        let columns = transforms.iter().map(|(col_name, _)| col_name.clone()).collect();

        Some((rule_label, columns))
    }

    /// Returns global and table-local assertions in execution order.
    pub fn all_asserts(&self) -> Vec<AssertScope<'_>> {
        let mut asserts = Vec::new();

        for assert in &self.asserts {
            asserts.push(AssertScope::Global(assert));
        }

        for table in &self.tables {
            for assert in &table.asserts {
                asserts.push(AssertScope::Table(TableAssert { table, assert }));
            }
        }

        asserts
    }

    /// Finds a table config matching any of the given candidate names.
    /// First tries exact match (preserving backward compatibility),
    /// then tries wildcard patterns.
    pub fn find_table<T: AsRef<str>>(&self, names: &[T]) -> Option<&Table> {
        self.find_table_match(names).map(|(table, _)| table)
    }

    /// Like `find_table`, but also returns whether the match was via a wildcard pattern.
    fn find_table_match<T: AsRef<str>>(&self, names: &[T]) -> Option<(&Table, bool)> {
        // Pass 1: exact match (existing behavior)
        for name in names {
            let table = self.get_table(name.as_ref());
            if table.is_some() {
                return table.map(|t| (t, false));
            }
        }

        // Pass 2: pattern match — covers wildcard entries and `names` entries
        // (even non-wildcard `names` entries, since pass 1 only checks `name` field)
        for table_cfg in &self.tables {
            if table_cfg.names.is_none() && !table_cfg.has_wildcards() {
                continue;
            }
            let is_wild = |s: &str| s.contains('*') || s.contains('?');
            for pattern in table_cfg.patterns() {
                let matcher = WildMatch::new(pattern);
                for name in names {
                    if matcher.matches(name.as_ref()) {
                        return Some((table_cfg, is_wild(pattern)));
                    }
                }
            }
        }

        None
    }

    /// Registers resolved transforms for a discovered table, resolving wildcard
    /// table name patterns against actual table metadata.
    /// Called from the dumper after table/column metadata is known.
    pub fn register_table_transforms(
        &mut self,
        full_name: &str,
        short_name: &str,
        actual_columns: &[String],
    ) {
        // Skip if already resolved by fill_transform_map (exact table)
        if let Some(map) = &self.transform_map {
            if map.contains_key(full_name) || map.contains_key(short_name) {
                return;
            }
        }

        let names = [full_name, short_name];
        let table_match = self.find_table_match(&names).map(|(t, w)| (t.clone(), w));

        if let Some((cfg, matched_via_wildcard)) = table_match {
            let transform_list = if matched_via_wildcard {
                // Wildcard table: silently skip rules for columns that don't exist
                let explicit_rule_order = cfg.rule_order.clone().unwrap_or_default();
                let mut list: TransformList = cfg
                    .rules
                    .iter()
                    .filter(|(col, _)| actual_columns.contains(col))
                    .map(|(col, t)| (col.clone(), t.clone()))
                    .collect();
                list.sort_by_cached_key(|(key, _)| {
                    explicit_rule_order.iter().position(|i| i == key)
                });
                list
            } else {
                // Exact table (via names field): use all rules as-is
                cfg.transform_list()
            };

            if transform_list.is_empty() {
                return;
            }

            let map = self.transform_map.get_or_insert_with(HashMap::new);
            // Use the config name for exact entries, full_name for wildcard/names entries
            let key = if cfg.name.is_empty() || matched_via_wildcard {
                full_name.to_string()
            } else {
                cfg.name.clone()
            };
            map.insert(key, transform_list);
        }
    }

    fn preprocess(&mut self) {
        let mut init_ctx = TransformerInitContext::from_defaults(self.default.clone());

        // Assign extend templates to context
        if let Some(collection) = &self.templates {
            init_ctx.template_collection = collection.clone();
        }

        for table in self.tables.iter_mut() {
            for (_name, rule) in table.rules.iter_mut() {
                rule.init(&init_ctx);
            }
        }

        self.fill_transform_map();
    }

    fn fill_transform_map(&mut self) {
        let mut map = HashMap::with_capacity(self.tables.len());
        for table in &self.tables {
            // Only pre-resolve entries with an exact table name.
            // Wildcard/names entries are deferred to register_table_transforms()
            // at dump time when actual table metadata is available.
            if table.names.is_some() || table.has_wildcards() {
                continue;
            }
            map.insert(table.name.clone(), table.transform_list());
        }

        self.transform_map = Some(map);
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::{transformers::PersonNameTransformer, LocaleConfig};
    use serde_json::json;

    #[test]
    fn set_defaults() {
        let config = r#"
            tables:
              - name: user
                rules:
                  name:
                    person_name: {}
                  alias:
                    person_name:
                      locale: EN
            default:
              locale: RU
            "#;

        let s = Settings::from_yaml(config).unwrap();
        let rules = &s.tables.first().unwrap().rules;

        assert_eq!(
            rules["name"],
            Transformers::PersonName(PersonNameTransformer {
                locale: Some(LocaleConfig::RU)
            })
        );
        assert_eq!(
            rules["alias"],
            Transformers::PersonName(PersonNameTransformer {
                locale: Some(LocaleConfig::EN)
            })
        );
    }

    #[test]
    fn find_table() {
        let config = r#"
            tables:
              - name: companies
                rules:
                  name:
                    company_name: {}
              - name: users
                rules:
                  name:
                    person_name: {}
              - name: other_schema.users
                rules:
                  other_name:
                    person_name: {}
            "#;
        let s = Settings::from_yaml(config).unwrap();

        let t = s.find_table(&["some_table"]);
        assert!(t.is_none());

        let t = s.find_table(&["some_table", "users"]);
        assert_eq!(t.unwrap().name, "users");

        let t = s.find_table(&["users", "other_schema.users"]);
        assert_eq!(t.unwrap().name, "users");

        let t = s.find_table(&["other_schema.users", "users"]);
        assert_eq!(t.unwrap().name, "other_schema.users");
    }

    mod transformers_for {
        use super::*;

        fn rule_names(s: &Settings, t: &str) -> Vec<String> {
            s.transformers_for(t)
                .unwrap()
                .iter()
                .map(|(name, _)| name.to_string())
                .collect()
        }

        #[test]
        fn order() {
            let config = r#"
                tables:
                  - name: table1
                    rule_order:
                      - greeting
                      - options
                    rules:
                      options:
                        template:
                          format: "{greeting: \"{{ final.greeting }}\"}"
                      greeting:
                        template:
                          format: "dear {{ final.first_name }} {{ final.last_name }}"
                      first_name:
                        first_name: {}
                      last_name:
                        last_name: {}
                  - name: table2
                    rules:
                      first_name:
                        first_name: {}
                      last_name:
                        last_name: {}
                "#;
            let s = Settings::from_yaml(config).unwrap();

            let names = rule_names(&s, "table1");
            assert_eq!(names.len(), 4);
            assert!(names.contains(&"first_name".to_string()));
            assert!(names.contains(&"last_name".to_string()));
            assert_eq!(names[2], "greeting");
            assert_eq!(names[3], "options");

            let names = rule_names(&s, "table2");
            assert_eq!(names.len(), 2);
            assert!(names.contains(&"first_name".to_string()));
            assert!(names.contains(&"last_name".to_string()));

            assert_eq!(s.transformers_for("table3"), None);
        }
    }

    mod templates_for {
        use super::*;

        fn get_raw_templates(s: &Settings) -> Vec<String> {
            s.templates
                .clone()
                .unwrap()
                .raw
                .unwrap()
                .keys()
                .map(|key| key.to_string())
                .collect()
        }

        fn get_files_templates(s: &Settings) -> Vec<String> {
            s.templates.clone().unwrap().files.unwrap()
        }

        #[test]
        fn read_templates() {
            let config = r#"
                tables: []
                templates:
                  raw:
                    template1: "template1"
                    template2: |
                      template2-line-1
                      template2-line-2
                  files:
                    - ./templates/path1
                    - ./templates/path2
                "#;
            let s = Settings::from_yaml(config).unwrap();

            assert_eq!(get_raw_templates(&s).len(), 2);
            assert_eq!(get_files_templates(&s).len(), 2);
        }
    }

    #[test]
    fn collects_global_and_table_asserts() {
        let config = r#"
            asserts:
              - name: global_check
                sql: select count(*) from users
                expect:
                  eq: 1
            tables:
              - name: users
                rules: {}
                asserts:
                  - name: table_check
                    sql: select 1 where false
                    expect: no_rows
            "#;

        let settings = Settings::from_yaml(config).unwrap();
        let asserts = settings.all_asserts();

        assert_eq!(asserts.len(), 2);
        assert_eq!(asserts[0].assert().name, "global_check");
        assert_eq!(asserts[0].scope_name(), "global");
        assert_eq!(asserts[1].assert().name, "table_check");
        assert_eq!(asserts[1].scope_name(), "users");
    }

    #[test]
    fn parses_global_asserts() {
        let config = r#"
            asserts:
              - name: users_count
                sql: select count(*) from users
                expect:
                  eq: 0
            tables: []
            "#;

        let settings = Settings::from_yaml(config).unwrap();

        assert_eq!(settings.asserts.len(), 1);
        assert_eq!(
            settings.asserts[0].expect,
            AssertExpectation::Scalar(Box::new(ScalarExpectations {
                eq: Some(json!(0)),
                not_eq: None,
                gt: None,
                gte: None,
                lt: None,
                lte: None,
            }))
        );
    }

    mod validation {
        use super::*;

        #[test]
        fn rejects_both_name_and_names() {
            let config = r#"
                tables:
                  - name: "public.users"
                    names:
                      - "A.*"
                    rules:
                      email:
                        person_name: {}
                "#;
            let err = Settings::from_yaml(config).unwrap_err();
            assert!(
                err.to_string().contains("cannot specify both"),
                "Expected 'cannot specify both' error, got: {}",
                err
            );
        }

        #[test]
        fn rejects_neither_name_nor_names() {
            let config = r#"
                tables:
                  - rules:
                      email:
                        person_name: {}
                "#;
            let err = Settings::from_yaml(config).unwrap_err();
            assert!(
                err.to_string().contains("must specify either"),
                "Expected 'must specify either' error, got: {}",
                err
            );
        }

        #[test]
        fn rejects_empty_names_list() {
            let config = r#"
                tables:
                  - names: []
                    rules:
                      email:
                        person_name: {}
                "#;
            let err = Settings::from_yaml(config).unwrap_err();
            assert!(
                err.to_string().contains("must specify either"),
                "Expected 'must specify either' error, got: {}",
                err
            );
        }
    }

    mod wildcard_table_matching {
        use super::*;

        #[test]
        fn wildcard_name_matches() {
            let config = r#"
                tables:
                  - name: "public.*"
                    rules:
                      email:
                        person_name: {}
                "#;
            let s = Settings::from_yaml(config).unwrap();

            let t = s.find_table(&["public.users"]);
            assert!(t.is_some());

            let t = s.find_table(&["other.users"]);
            assert!(t.is_none());
        }

        #[test]
        fn exact_match_beats_wildcard() {
            let config = r#"
                tables:
                  - name: "public.*"
                    rules:
                      email:
                        person_name: {}
                  - name: public.users
                    rules:
                      email:
                        first_name: {}
                "#;
            let s = Settings::from_yaml(config).unwrap();

            // Exact match should win
            let t = s.find_table(&["public.users", "users"]);
            assert_eq!(t.unwrap().name, "public.users");

            // Wildcard should match other tables
            let t = s.find_table(&["public.orders", "orders"]);
            assert_eq!(t.unwrap().name, "public.*");
        }

        #[test]
        fn names_field_with_multiple_patterns() {
            let config = r#"
                tables:
                  - names:
                      - "A.*"
                      - "B.*"
                    rules:
                      email:
                        person_name: {}
                "#;
            let s = Settings::from_yaml(config).unwrap();

            assert!(s.find_table(&["A.users"]).is_some());
            assert!(s.find_table(&["B.orders"]).is_some());
            assert!(s.find_table(&["C.stuff"]).is_none());
        }

        #[test]
        fn first_wildcard_entry_wins() {
            let config = r#"
                tables:
                  - name: "public.*"
                    rules:
                      email:
                        person_name: {}
                  - name: "public.u*"
                    rules:
                      email:
                        first_name: {}
                "#;
            let s = Settings::from_yaml(config).unwrap();

            // First wildcard entry should win
            let t = s.find_table(&["public.users"]);
            assert_eq!(t.unwrap().name, "public.*");
        }

        #[test]
        fn names_field_with_exact_values() {
            let config = r#"
                tables:
                  - names:
                      - "A.users"
                      - "B.orders"
                    rules:
                      email:
                        person_name: {}
                "#;
            let s = Settings::from_yaml(config).unwrap();

            assert!(s.find_table(&["A.users"]).is_some());
            assert!(s.find_table(&["B.orders"]).is_some());
            assert!(s.find_table(&["A.orders"]).is_none());
        }

        #[test]
        fn backward_compat_no_wildcards() {
            let config = r#"
                tables:
                  - name: users
                    rules:
                      name:
                        person_name: {}
                "#;
            let s = Settings::from_yaml(config).unwrap();

            let t = s.find_table(&["public.users", "users"]);
            assert_eq!(t.unwrap().name, "users");
        }
    }

    mod register_table_transforms_tests {
        use super::*;

        fn transform_keys(s: &Settings, table: &str) -> Vec<String> {
            match s.transformers_for(table) {
                Some(list) => list.iter().map(|(name, _)| name.clone()).collect(),
                None => vec![],
            }
        }

        #[test]
        fn exact_table_not_overwritten() {
            let config = r#"
                tables:
                  - name: users
                    rules:
                      name:
                        person_name: {}
                  - name: "public.*"
                    rules:
                      email:
                        first_name: {}
                "#;
            let mut s = Settings::from_yaml(config).unwrap();

            // Exact table already in transform_map from fill_transform_map
            let keys = transform_keys(&s, "users");
            assert_eq!(keys.len(), 1);
            assert!(keys.contains(&"name".to_string()));

            // register_table_transforms should not overwrite it
            s.register_table_transforms(
                "public.users",
                "users",
                &["name".to_string(), "email".to_string()],
            );

            // Still the exact-match entry
            let keys = transform_keys(&s, "users");
            assert_eq!(keys.len(), 1);
            assert!(keys.contains(&"name".to_string()));
        }

        #[test]
        fn names_field_registers_correctly() {
            let config = r#"
                tables:
                  - names:
                      - "A.*"
                      - "B.*"
                    rules:
                      email:
                        person_name: {}
                "#;
            let mut s = Settings::from_yaml(config).unwrap();

            s.register_table_transforms(
                "A.users",
                "users",
                &["id".to_string(), "email".to_string()],
            );
            s.register_table_transforms(
                "B.orders",
                "orders",
                &["id".to_string(), "email".to_string()],
            );

            let keys_a = transform_keys(&s, "A.users");
            assert_eq!(keys_a.len(), 1);
            assert!(keys_a.contains(&"email".to_string()));

            let keys_b = transform_keys(&s, "B.orders");
            assert_eq!(keys_b.len(), 1);
            assert!(keys_b.contains(&"email".to_string()));
        }

        #[test]
        fn wildcard_table_exact_column_missing_silently_skipped() {
            let config = r#"
                tables:
                  - name: "public.*"
                    rules:
                      email:
                        person_name: {}
                      phone:
                        person_name: {}
                "#;
            let mut s = Settings::from_yaml(config).unwrap();

            // Table has email but not phone — phone should be silently skipped
            s.register_table_transforms(
                "public.logs",
                "logs",
                &["id".to_string(), "email".to_string()],
            );

            let keys = transform_keys(&s, "public.logs");
            assert_eq!(keys.len(), 1);
            assert!(keys.contains(&"email".to_string()));
            assert!(!keys.contains(&"phone".to_string()));
        }

        #[test]
        fn names_mixed_exact_and_wildcard_strict_for_exact_match() {
            let config = r#"
                tables:
                  - names:
                      - "_sqlx_migrations"
                      - "ok*"
                    rules:
                      description:
                        person_name: {}
                "#;
            let mut s = Settings::from_yaml(config).unwrap();

            // _sqlx_migrations matches via exact pattern — should keep
            // missing exact columns (strict mode)
            s.register_table_transforms(
                "public._sqlx_migrations",
                "_sqlx_migrations",
                &["id".to_string(), "version".to_string()],
            );

            // "description" doesn't exist but should be kept (exact table match)
            let keys = transform_keys(&s, "public._sqlx_migrations");
            assert_eq!(keys.len(), 1);
            assert!(keys.contains(&"description".to_string()));
        }

        #[test]
        fn names_mixed_exact_and_wildcard_lenient_for_wildcard_match() {
            let config = r#"
                tables:
                  - names:
                      - "_sqlx_migrations"
                      - "ok*"
                    rules:
                      description:
                        person_name: {}
                "#;
            let mut s = Settings::from_yaml(config).unwrap();

            // ok_stuff matches via wildcard pattern — should silently skip
            // missing columns (lenient mode)
            s.register_table_transforms(
                "public.ok_stuff",
                "ok_stuff",
                &["id".to_string(), "name".to_string()],
            );

            // "description" doesn't exist and should be skipped (wildcard table match)
            assert!(s.transformers_for("public.ok_stuff").is_none());
        }

        #[test]
        fn overlapping_wildcards_first_entry_wins() {
            let config = r#"
                tables:
                  - name: "public.*"
                    rules:
                      email:
                        person_name: {}
                  - name: "*users*"
                    rules:
                      phone:
                        person_name: {}
                "#;
            let mut s = Settings::from_yaml(config).unwrap();

            // Both patterns match public.users — first entry (public.*) should win
            s.register_table_transforms(
                "public.users",
                "users",
                &["email".to_string(), "phone".to_string()],
            );

            let keys = transform_keys(&s, "public.users");
            // Should have email (from public.*), not phone (from *users*)
            assert!(keys.contains(&"email".to_string()));
            assert!(!keys.contains(&"phone".to_string()));
        }

        #[test]
        fn names_field_exact_values_registers_correctly() {
            let config = r#"
                tables:
                  - names:
                      - "A.users"
                      - "B.users"
                    rules:
                      email:
                        person_name: {}
                "#;
            let mut s = Settings::from_yaml(config).unwrap();

            // Should not be in transform_map from fill_transform_map
            assert!(s.transformers_for("A.users").is_none());

            s.register_table_transforms(
                "A.users",
                "users",
                &["id".to_string(), "email".to_string()],
            );

            let keys = transform_keys(&s, "A.users");
            assert_eq!(keys.len(), 1);
            assert!(keys.contains(&"email".to_string()));
        }
    }
}
