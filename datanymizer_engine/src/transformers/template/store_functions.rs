use crate::store::KeyValueStore;
use std::collections::HashMap;
use tera::{Function, Tera, Value};

pub fn register<S: 'static + KeyValueStore>(t: &mut Tera, store: S) {
    t.register_function("store_read", read(store.clone()));
    t.register_function("store_write", write(store.clone()));
    t.register_function("store_force_write", force_write(store.clone()));
    t.register_function("store_add_float", add_float(store.clone()));
    t.register_function("store_add_int", add_int(store));
}

fn read<S: KeyValueStore>(store: S) -> impl Function {
    Box::new(
        move |args: &HashMap<String, Value>| -> tera::Result<Value> {
            match args.get("key") {
                Some(key) => {
                    let key = tera::from_value::<String>(key.clone())?;
                    match store.read_value(&key) {
                        Some(value) => Ok(value),
                        None => Err(format!("No such key {}", key).into()),
                    }
                }
                None => Err("No key argument".into()),
            }
        },
    )
}

fn write<S: KeyValueStore>(store: S) -> impl Function {
    Box::new(
        move |args: &HashMap<String, Value>| -> tera::Result<Value> {
            match args.get("key") {
                Some(key) => {
                    let key = tera::from_value::<String>(key.clone())?;
                    match args.get("value") {
                        Some(value) => {
                            store.write_value(key, value.clone());
                            Ok(Value::Null)
                        }
                        None => Err("No value argument".into()),
                    }
                }
                None => Err("No key argument".into()),
            }
        },
    )
}

fn force_write<S: KeyValueStore>(store: S) -> impl Function {
    Box::new(
        move |args: &HashMap<String, Value>| -> tera::Result<Value> {
            match args.get("key") {
                Some(key) => {
                    let key = tera::from_value::<String>(key.clone())?;
                    match args.get("value") {
                        Some(value) => {
                            store.force_write_value(key, value.clone());
                            Ok(Value::Null)
                        }
                        None => Err("No value argument".into()),
                    }
                }
                None => Err("No key argument".into()),
            }
        },
    )
}

fn add_int<S: KeyValueStore>(store: S) -> impl Function {
    Box::new(
        move |args: &HashMap<String, Value>| -> tera::Result<Value> {
            match args.get("key") {
                Some(key) => {
                    let key = tera::from_value::<String>(key.clone())?;
                    match args.get("value") {
                        Some(value) => {
                            store.add_int_value(key, value.clone());
                            Ok(Value::Null)
                        }
                        None => Err("No value argument".into()),
                    }
                }
                None => Err("No key argument".into()),
            }
        },
    )
}

fn add_float<S: KeyValueStore>(store: S) -> impl Function {
    Box::new(
        move |args: &HashMap<String, Value>| -> tera::Result<Value> {
            match args.get("key") {
                Some(key) => {
                    let key = tera::from_value::<String>(key.clone())?;
                    match args.get("value") {
                        Some(value) => {
                            store.add_float_value(key, value.clone());
                            Ok(Value::Null)
                        }
                        None => Err("No value argument".into()),
                    }
                }
                None => Err("No key argument".into()),
            }
        },
    )
}
