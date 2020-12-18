use once_cell::sync::Lazy;
use std::collections::{hash_map::DefaultHasher, HashSet};
use std::hash::{Hash, Hasher};
use std::sync::Mutex;

static GLOBAL_DATA: Lazy<Mutex<HashSet<u64>>> = Lazy::new(|| Mutex::new(HashSet::new()));

pub(crate) fn add_to_collector(name: &str, value: &str) -> bool {
    if let Ok(mut counter) = GLOBAL_DATA.lock() {
        let mut hasher = DefaultHasher::new();
        format!("{}.{}", name, value).hash(&mut hasher);
        counter.insert(hasher.finish())
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uniq_values() {
        let name = "uniq_collector.uniq_values.name";

        assert!(add_to_collector(name, "val1"));
        assert!(add_to_collector(name, "val2"));
    }

    #[test]
    fn same_values() {
        let name = "uniq_collector.same_values.name";

        assert!(add_to_collector(name, "val"));
        assert!(!add_to_collector(name, "val"));
    }

    #[test]
    fn same_values_with_different_names() {
        let name1 = "uniq_collector.same_values_with_different_names.name1";
        let name2 = "uniq_collector.same_values_with_different_names.name2";

        assert!(add_to_collector(name1, "val"));
        assert!(add_to_collector(name2, "val"));
    }
}
