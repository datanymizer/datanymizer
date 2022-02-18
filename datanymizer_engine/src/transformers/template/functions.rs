use crate::store::KeyValueStore;
use rand::Rng;
use std::collections::HashMap;
use tera::{Function, Tera, Value};

pub fn register<S: 'static + KeyValueStore>(t: &mut Tera, store: S) {
    t.register_function("store_read", read(store.clone()));
    t.register_function("store_write", write(store.clone()));
    t.register_function("store_force_write", force_write(store.clone()));
    t.register_function("store_inc", inc(store));

    t.register_function("get_random", get_random);
}

fn read<S: KeyValueStore>(store: S) -> impl Function {
    Box::new(
        move |args: &HashMap<String, Value>| -> tera::Result<Value> {
            match args.get("key") {
                Some(key) => {
                    let key = tera::from_value::<String>(key.clone())?;
                    match store.read_value(&key) {
                        Ok(value) => match value {
                            Some(value) => Ok(value),
                            None => match args.get("default") {
                                Some(default) => Ok(default.clone()),
                                None => {
                                    Err(format!("No such key {} and no default value", key).into())
                                }
                            },
                        },
                        Err(e) => Err(e.to_string().into()),
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
                        Some(value) => match store.write_value(key, value.clone()) {
                            Ok(_) => Ok(Value::Null),
                            Err(e) => Err(e.to_string().into()),
                        },
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
                        Some(value) => match store.force_write_value(key, value.clone()) {
                            Ok(_) => Ok(Value::Null),
                            Err(e) => Err(e.to_string().into()),
                        },
                        None => Err("No value argument".into()),
                    }
                }
                None => Err("No key argument".into()),
            }
        },
    )
}

fn inc<S: KeyValueStore>(store: S) -> impl Function {
    Box::new(
        move |args: &HashMap<String, Value>| -> tera::Result<Value> {
            match args.get("key") {
                Some(key) => {
                    let key = tera::from_value::<String>(key.clone())?;
                    match args.get("value") {
                        Some(value) => match store.inc_value(key, value.clone()) {
                            Ok(_) => Ok(Value::Null),
                            Err(e) => Err(e.to_string().into()),
                        },
                        None => Err("No value argument".into()),
                    }
                }
                None => Err("No key argument".into()),
            }
        },
    )
}

pub fn get_random(args: &HashMap<String, Value>) -> tera::Result<Value> {
    let start = match args.get("start") {
        Some(val) => match tera::from_value::<i32>(val.clone()) {
            Ok(v) => v,
            Err(_) => {
                return Err(tera::Error::msg(format!(
                    "Function `get_random` received start={} but `start` can only be a boolean",
                    val
                )));
            }
        },
        None => 0,
    };

    let end = match args.get("end") {
        Some(val) => match tera::from_value::<i32>(val.clone()) {
            Ok(v) => v,
            Err(_) => {
                return Err(tera::Error::msg(format!(
                    "Function `get_random` received end={} but `end` can only be a boolean",
                    val
                )));
            }
        },
        None => {
            return Err(tera::Error::msg(
                "Function `get_random` didn't receive an `end` argument",
            ))
        }
    };
    let mut rng = rand::thread_rng();
    let res = rng.gen_range(start..end);

    Ok(Value::Number(res.into()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::TransformerInitContext;
    use tera::Context;

    #[test]
    fn register_functions() {
        let store = TransformerInitContext::default().template_store;
        let mut t = Tera::default();

        let read_template = "{{ store_read(key='key') }}";
        let read_default_template = "{{ store_read(key='no_key', default='def') }}";
        let write_template = "{{ store_write(key='key', value='abc') }} \
          {{ store_force_write(key='key2', value='cde') }} \
          {{ store_inc(key='key3', value=1) }}";

        register(&mut t, store.clone());
        t.add_raw_template("read", &read_template).unwrap();
        t.add_raw_template("read_default", &read_default_template)
            .unwrap();
        t.add_raw_template("write", &write_template).unwrap();

        t.render("write", &Context::new()).unwrap();
        assert_eq!(t.render("read", &Context::new()).unwrap(), "abc");
        assert_eq!(t.render("read_default", &Context::new()).unwrap(), "def");

        assert_eq!(store.read_value("key2").unwrap().unwrap(), "cde");
        assert_eq!(store.read_value("key3").unwrap().unwrap(), 1);
    }
}
