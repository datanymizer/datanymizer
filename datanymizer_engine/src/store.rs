use serde_json::Value;
use std::{
    collections::{hash_map::Entry, HashMap},
    sync::{Arc, RwLock},
};

pub trait KeyValueStore: Clone + Sync + Send {
    fn read_value(&self, key: &str) -> Option<Value>;

    fn write_value(&self, key: String, value: Value);

    fn force_write_value(&self, key: String, value: Value);

    fn add_int_value(&self, key: String, value: Value);

    fn add_float_value(&self, key: String, value: Value);
}

impl KeyValueStore for Arc<RwLock<HashMap<String, Value>>> {
    fn read_value(&self, key: &str) -> Option<Value> {
        let map = self.as_ref().read().expect("Sync read error");
        map.get(key).cloned()
    }

    fn write_value(&self, key: String, value: Value) {
        let mut map = self.as_ref().write().expect("Sync write error");
        match map.entry(key) {
            Entry::Occupied(e) => panic!("Can't overwrite the key {}", e.key()),
            Entry::Vacant(e) => e.insert(value),
        };
    }

    fn force_write_value(&self, key: String, value: Value) {
        let mut map = self.as_ref().write().expect("Sync write error");
        map.insert(key, value);
    }

    fn add_int_value(&self, key: String, value: Value) {
        let mut map = self.as_ref().write().expect("Sync write error");
        match map.entry(key) {
            Entry::Occupied(mut e) => {
                if !(e.get().is_i64() && value.is_i64()) {
                    panic!("Can't add to int {}", e.key())
                }
                e.insert(Value::from(
                    e.get().as_i64().unwrap() + value.as_i64().unwrap(),
                ));
            }
            Entry::Vacant(e) => {
                if !value.is_i64() {
                    panic!("Can't add to int {}", e.key())
                }
                e.insert(value);
            }
        };
    }

    fn add_float_value(&self, key: String, value: Value) {
        let mut map = self.as_ref().write().expect("Sync write error");
        match map.entry(key) {
            Entry::Occupied(mut e) => {
                if !(e.get().is_f64() && value.is_f64()) {
                    panic!("Can't add to float {}", e.key())
                }
                e.insert(Value::from(
                    e.get().as_f64().unwrap() + value.as_f64().unwrap(),
                ));
            }
            Entry::Vacant(e) => {
                if !value.is_f64() {
                    panic!("Can't add to float {}", e.key())
                }
                e.insert(value);
            }
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn store() -> Arc<RwLock<HashMap<String, Value>>> {
        Arc::new(RwLock::new(HashMap::new()))
    }

    #[test]
    fn read_and_write() {
        let key = "key";
        let store = store();

        assert_eq!(store.read_value(key), None);

        store.force_write_value(String::from(key), Value::from("123"));
        assert_eq!(store.read_value(key).unwrap(), "123");

        store.force_write_value(String::from(key), Value::from("321"));
        assert_eq!(store.read_value(key).unwrap(), "321");
    }

    #[test]
    #[should_panic(expected = "Can't overwrite the key some_key")]
    fn no_overwrite() {
        let key = "some_key";
        let store = store();

        assert_eq!(store.read_value(key), None);

        store.write_value(String::from(key), Value::from("abc"));
        assert_eq!(store.read_value(key).unwrap(), "abc");

        store.write_value(String::from(key), Value::from("abc"));
    }

    #[test]
    fn add_int_value() {
        let key = "some_key";
        let store = store();
        let a = Value::from(2);
        let b = Value::from(3);

        store.add_int_value(String::from(key), a);
        assert_eq!(store.read_value(key).unwrap(), 2);

        store.add_int_value(String::from(key), b);
        assert_eq!(store.read_value(key).unwrap(), 5);
    }

    #[test]
    fn add_float_value() {
        let key = "some_key";
        let store = store();
        let a = Value::from(1.0);
        let b = Value::from(2.0);

        store.add_float_value(String::from(key), a);
        assert_eq!(store.read_value(key).unwrap(), 1.0);

        store.add_float_value(String::from(key), b);
        assert_eq!(store.read_value(key).unwrap(), 3.0);
    }
}
