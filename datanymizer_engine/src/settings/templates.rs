use serde::Deserialize;

use std::collections::HashMap;

/// Shared templates for all instances of `TemplateTransformer`
#[derive(Debug, Deserialize, Clone)]
pub struct TemplatesCollection {
    /// Raw named templates.
    /// Example:
    ///
    /// ```yaml
    /// # ...
    /// templates:
    ///   raw:
    ///     debug_me: "old_value: {{ _0 }}"
    ///     multiline_example: |
    ///       {
    ///         "old_value": "{{prev.name}}",
    ///         "new_value": "{{final.name}}",
    ///       }
    /// ```
    pub raw: Option<HashMap<String, String>>,
    /// Files with templates for import into templates
    /// Example:
    ///
    /// ```yaml
    /// # ...
    /// templates:
    ///   files:
    ///     - ./templates/base.html
    ///     - ./templates/user_data_json
    pub files: Option<Vec<String>>,
}

/// Default value for `TemplatesCollection`
impl Default for TemplatesCollection {
    fn default() -> Self {
        Self {
            raw: None,
            files: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod templates_list {
        use super::*;

        #[test]
        fn empty_defaults() {
            let t = TemplatesCollection::default();

            assert_eq!(t.raw, None);
            assert_eq!(t.files, None);
        }


        #[test]
        fn ext_templates() {
            let config = r#"
            raw:
              template1: "{{ _0 }}"
              template2: |
                new_value: {{ final.name }}
            files:
              - ./templates/base.html
              - ./templates/base.html
            "#;
            let t: TemplatesCollection = serde_yaml::from_str(config).unwrap();

            assert_eq!(t.files.unwrap().len(), 2);
            assert_eq!(t.raw.unwrap().keys().len(), 2);
        }
    }
}
