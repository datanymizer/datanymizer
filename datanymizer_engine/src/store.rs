use anyhow::{anyhow, bail, Result};
use serde_json::Value;
use std::{
    collections::{hash_map::Entry, HashMap},
    sync::{Arc, RwLock},
};

pub trait KeyValueStore: Clone + Sync + Send {
    fn read_value(&self, key: &str) -> Result<Option<Value>>;

    fn write_value(&self, key: String, value: Value) -> Result<()>;

    fn force_write_value(&self, key: String, value: Value) -> Result<()>;

    fn inc_value(&self, key: String, value: Value) -> Result<()>;
}

impl KeyValueStore for Arc<RwLock<HashMap<String, Value>>> {
    fn read_value(&self, key: &str) -> Result<Option<Value>> {
        match self.as_ref().read() {
            Ok(map) => Ok(map.get(key).cloned()),
            Err(_) => Err(anyhow!("Sync read error")),
        }
    }

    fn write_value(&self, key: String, value: Value) -> Result<()> {
        match self.as_ref().write() {
            Ok(mut map) => match map.entry(key) {
                Entry::Occupied(e) => Err(anyhow!("Can't overwrite the key {}", e.key())),
                Entry::Vacant(e) => {
                    e.insert(value);
                    Ok(())
                }
            },
            Err(_) => Err(anyhow!("Sync write error")),
        }
    }

    fn force_write_value(&self, key: String, value: Value) -> Result<()> {
        match self.as_ref().write() {
            Ok(mut map) => {
                map.insert(key, value);
                Ok(())
            }
            Err(_) => Err(anyhow!("Sync write error")),
        }
    }

    fn inc_value(&self, key: String, value: Value) -> Result<()> {
        match self.as_ref().write() {
            Ok(mut map) => {
                match map.entry(key) {
                    Entry::Occupied(mut e) => {
                        let entry_value = e.get();
                        let sum;
                        if entry_value.is_i64() && value.is_i64() {
                            sum = Value::from(
                                entry_value.as_i64().unwrap() + value.as_i64().unwrap(),
                            );
                        } else if let (Some(a), Some(b)) = (entry_value.as_f64(), value.as_f64()) {
                            sum = Value::from(a + b);
                        } else {
                            bail!(
                                "Can't increment a value for the key {} (not a number)",
                                e.key()
                            );
                        };

                        e.insert(sum);
                    }
                    Entry::Vacant(e) => {
                        if !value.is_number() {
                            bail!(
                                "Can't increment a value for the key {} (not a number)",
                                e.key()
                            )
                        }
                        e.insert(value);
                    }
                };

                Ok(())
            }
            Err(_) => Err(anyhow!("Sync write error")),
        }
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

        assert_eq!(store.read_value(key).unwrap(), None);

        store
            .force_write_value(String::from(key), Value::from("123"))
            .unwrap();
        assert_eq!(store.read_value(key).unwrap().unwrap(), "123");

        store
            .force_write_value(String::from(key), Value::from("321"))
            .unwrap();
        assert_eq!(store.read_value(key).unwrap().unwrap(), "321");
    }

    #[test]
    fn no_overwrite() {
        let key = "some_key";
        let store = store();

        assert_eq!(store.read_value(key).unwrap(), None);

        store
            .write_value(String::from(key), Value::from("abc"))
            .unwrap();
        assert_eq!(store.read_value(key).unwrap().unwrap(), "abc");

        let result = store.write_value(String::from(key), Value::from("abc"));
        assert_eq!(
            result.err().unwrap().to_string(),
            "Can't overwrite the key some_key"
        );
    }

    mod inc_value {
        use super::*;

        #[test]
        fn both_int() {
            let key = "some_key";
            let store = store();

            store.inc_value(String::from(key), Value::from(2)).unwrap();
            assert_eq!(store.read_value(key).unwrap().unwrap(), 2);

            store.inc_value(String::from(key), Value::from(3)).unwrap();
            assert_eq!(store.read_value(key).unwrap().unwrap(), 5);
        }

        #[test]
        fn both_float() {
            let key = "some_key";
            let store = store();

            store
                .inc_value(String::from(key), Value::from(2.0))
                .unwrap();
            assert_eq!(store.read_value(key).unwrap().unwrap(), 2.0);

            store
                .inc_value(String::from(key), Value::from(3.5))
                .unwrap();
            assert_eq!(store.read_value(key).unwrap().unwrap(), 5.5);
        }

        #[test]
        fn int_and_float() {
            let key = "some_key";
            let store = store();

            store.inc_value(String::from(key), Value::from(2)).unwrap();
            assert_eq!(store.read_value(key).unwrap().unwrap(), 2);

            store
                .inc_value(String::from(key), Value::from(3.5))
                .unwrap();
            assert_eq!(store.read_value(key).unwrap().unwrap(), 5.5);
        }

        #[test]
        fn not_a_number() {
            let key = "some_key";
            let store = store();

            assert_eq!(store.read_value(key).unwrap(), None);

            store.inc_value(String::from(key), Value::from(10)).unwrap();
            assert_eq!(store.read_value(key).unwrap().unwrap(), 10);

            let result = store.inc_value(String::from(key), Value::from("abc"));
            assert_eq!(
                result.err().unwrap().to_string(),
                "Can't increment a value for the key some_key (not a number)"
            );
        }

        #[test]
        fn not_a_number_first() {
            let key = "some_key";
            let store = store();

            assert_eq!(store.read_value(key).unwrap(), None);

            let result = store.inc_value(String::from(key), Value::from("abc"));
            assert_eq!(
                result.err().unwrap().to_string(),
                "Can't increment a value for the key some_key (not a number)"
            );
        }
    }
}
