use super::TransformList;
use crate::Transformers;
use serde::Deserialize;
use std::collections::HashMap;

type Rules = HashMap<String, Transformers>;

#[derive(Debug, Deserialize, Clone)]
pub struct Query {
    /// SQL limit
    pub limit: Option<usize>,
    /// SQL condition (WHERE) for dumping
    pub dump_condition: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Table {
    /// Table name
    pub name: String,
    /// Rule set for columns
    pub rules: Rules,
    /// Order of applying rules. All rules not listed are placed at the beginning
    pub rule_order: Option<Vec<String>>,
    /// Limit and conditions for the dumping query
    pub query: Option<Query>,
}

impl Table {
    pub fn transform_list(&self) -> TransformList {
        let explicit_rule_order = self.rule_order.clone().unwrap_or_default();
        let mut transform_list: TransformList = self
            .rules
            .iter()
            .map(|(key, ts)| (key.clone(), ts.clone()))
            .collect();
        transform_list
            .sort_by_cached_key(|(key, _)| explicit_rule_order.iter().position(|i| i == key));

        transform_list
    }
}
