use std::collections::HashMap;

use bcrypt::{DEFAULT_COST, hash};
use serde_json::{from_value, Value};
use tera::{Tera, Result, Error, try_get_value};

pub fn register(t: &mut Tera) {
    t.register_filter("bcrypt_hash", bcrypt_hash);
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
    use tera::Context;
    use bcrypt::verify;

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
}
