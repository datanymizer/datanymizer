use once_cell::sync::Lazy;
use std::{collections::HashSet, sync::Mutex};
static GLOBAL_DATA: Lazy<Mutex<HashSet<String>>> = Lazy::new(|| Mutex::new(HashSet::new()));

pub fn add_to_collector(item: String) -> bool {
    if let Ok(mut counter) = GLOBAL_DATA.lock() {
        counter.insert(item)
    } else {
        false
    }
}
