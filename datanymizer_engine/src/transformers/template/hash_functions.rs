use std::collections::HashMap;

use bcrypt::{hash, DEFAULT_COST};
use serde_json::{from_value, Value};
use tera::{try_get_value, Error, Result, Tera};

use sha2::{Digest, Sha256};

pub fn register(t: &mut Tera) {
    t.register_filter("bcrypt_hash", bcrypt_hash);
    t.register_filter("sha256", sha256);
}

/// Sha256 hash function
/// You can generate salted sha256 hashes with it
///
/// # Examples
///
/// ```yaml
/// #...
/// rules:
///   some_value:
///     template:
///       format: "{{ _0 | sha256 }}"
///
///   some_value_more_confidential:
///     template:
///       format: "{{ _0 | sha256(rounds=10, salt='someverysecret') }}"
///
///   or_some_other_value:
///     template:
///       format: "{{ _0 | sha256(rounds=10, salt=secret_salt) }}"
///
///# you can concatenate this section in CI to avoid literal secrets
/// globals:
///   secret_salt: someverysecret
/// ```
fn sha256(value: &Value, args: &HashMap<String, Value>) -> Result<Value> {
    let rounds: u32 = match args.get("rounds") {
        Some(val) => match from_value::<u32>(val.clone()) {
            Ok(v) => v,
            Err(_) => {
                return Err(Error::msg(format!(
                    "Function `sha256` received rounds={} but `rounds` can only be an unsigned integer of 32bits",
                    val
                )));
            }
        },
        None => 1,
    };
    let salt: String = match args.get("salt") {
        Some(val) => match from_value::<String>(val.clone()) {
            Ok(v) => v,
            Err(_) => {
                return Err(Error::msg(format!(
                    "Function `sha256` received salt={} but `salt` can only be a string",
                    val
                )));
            }
        },
        None => "".to_string(),
    };

    let val = try_get_value!("sha256", "value", String, value);

    let mut hash: String = "".to_string();
    for i in 0..rounds {
        let mut sha256 = Sha256::new();
        if i == 0 {
            sha256.update(salt.clone());
            sha256.update(val.clone());
        } else {
            sha256.update(salt.clone());
            sha256.update(hash);
        }
        hash = format!("{:x}", sha256.finalize());
    }

    Ok(Value::from( hash.clone() ))
}

/// BCrypt hash function
/// You can generate password hashes with it
///
/// # Examples
///
/// ```yaml
/// #...
/// rules:
///   password_hash:
///     template:
///       format: "{{ _1 | bcrypt_hash }}"
///       rules:
///         - word: {}
///   password_hash_with_cost:
///     template:
///       format: "{{ _1 | bcrypt_hash(cost=10) }}"
///       rules:
///         - word: {}
/// ```
fn bcrypt_hash(value: &Value, args: &HashMap<String, Value>) -> Result<Value> {
    let cost: u32 = match args.get("cost") {
        Some(val) => match from_value::<u32>(val.clone()) {
            Ok(v) => v,
            Err(_) => {
                return Err(Error::msg(format!(
                    "Function `bcrypt_hash` recived cost={} but `bcrypt_hash` can only be a number",
                    val
                )));
            }
        },
        None => DEFAULT_COST,
    };

    let s = try_get_value!("bcrypt_hash", "value", String, value);
    hash(s, cost)
        .map(|hsh| hsh.into())
        .map_err(|err| Error::from(err.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use bcrypt::verify;
    use tera::Context;

    #[test]
    fn bcrypt_hash_default() {
        let mut t = Tera::default();
        let mut context = Context::new();
        let pass = "password";
        context.insert("password", pass);

        register(&mut t);
        let empty_template = "{{ password | bcrypt_hash }}";
        t.add_raw_template("empty_filter", &empty_template).unwrap();
        let real_value = t.render("empty_filter", &context).unwrap();
        assert!(verify(&pass, &real_value).unwrap());
    }

    #[test]
    fn bcrypt_hash_cost() {
        let mut t = Tera::default();
        let mut context = Context::new();
        let pass = "password";
        context.insert("password", pass);

        register(&mut t);
        let empty_template = "{{ password | bcrypt_hash(cost=10) }}";
        t.add_raw_template("empty_filter", &empty_template).unwrap();
        let real_value = t.render("empty_filter", &context).unwrap();
        assert!(verify(&pass, &real_value).unwrap());
    }

    #[test]
    fn sha256_default() {
        let mut t = Tera::default();
        let mut context = Context::new();
        let inp = "abc";
        context.insert("value", inp);
        register(&mut t);

        let template = "{{ value | sha256 }}";
        t.add_raw_template("empty_filter", &template).unwrap();

        let real_value = t.render("empty_filter", &context).unwrap();

        assert_eq!(
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad",
            &real_value,
        );
    }

    #[test]
    fn sha256_salt() {
        let mut t = Tera::default();
        let mut context = Context::new();
        let inp = "abc";
        context.insert("value", inp);
        register(&mut t);

        let template = "{{ value | sha256(salt='abcdef') }}";
        t.add_raw_template("empty_filter", &template).unwrap();

        let real_value = t.render("empty_filter", &context).unwrap();

        assert_eq!(
            "f9c8ac3b5d36269c49d88a3bec749842e0880625724d4896475639dc22ef5bf6",
            &real_value,
        );
    }

    #[test]
    fn sha256_salt_rounds() {
        let mut t = Tera::default();
        let mut context = Context::new();
        let inp = "abc";
        context.insert("value", inp);
        register(&mut t);

        let template = "{{ value | sha256(salt='abcdef', rounds=5) }}";
        t.add_raw_template("empty_filter", &template).unwrap();

        let real_value = t.render("empty_filter", &context).unwrap();

        assert_eq!(
            "a0436350051508bc76278569ddd1f5d7d1868d0403c9a6895abed9949e5cf0c2",
            &real_value,
        );
    }
    #[test]
    fn sha256_2salts_values_ne() {
        let mut t = Tera::default();
        let mut context = Context::new();
        let inp = "abc";
        context.insert("value", inp);
        register(&mut t);

        let template1 = "{{ value | sha256(salt='abc', rounds=5) }}";
        t.add_raw_template("filter1", &template1).unwrap();
        let real_value1 = t.render("filter1", &context).unwrap();

        let template2 = "{{ value | sha256(salt='def', rounds=5) }}";
        t.add_raw_template("filter2", &template2).unwrap();
        let real_value2 = t.render("filter2", &context).unwrap();

        assert_ne!(
            &real_value1,
            &real_value2,
        );
    }
}
