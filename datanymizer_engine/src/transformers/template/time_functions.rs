use serde_json::value::{from_value, to_value};
use std::collections::HashMap;
use tera::{Error, Result, Tera, Value};
use time::{format_description::well_known::Rfc3339, OffsetDateTime};

pub fn register(t: &mut Tera) {
    t.register_function("now", now);
}

fn now(args: &HashMap<String, Value>) -> Result<Value> {
    let utc = match args.get("utc") {
        Some(val) => match from_value::<bool>(val.clone()) {
            Ok(v) => v,
            Err(_) => {
                return Err(Error::msg(format!(
                    "Function `now` received utc={} but `utc` can only be a boolean",
                    val
                )));
            }
        },
        None => false,
    };
    let timestamp = match args.get("timestamp") {
        Some(val) => match from_value::<bool>(val.clone()) {
            Ok(v) => v,
            Err(_) => {
                return Err(Error::msg(format!(
                    "Function `now` received timestamp={} but `timestamp` can only be a boolean",
                    val
                )));
            }
        },
        None => false,
    };

    let datetime = if utc {
        OffsetDateTime::now_utc()
    } else {
        OffsetDateTime::now_local().unwrap_or_else(|_| OffsetDateTime::now_utc())
    };

    if timestamp {
        return Ok(to_value(datetime.unix_timestamp()).unwrap());
    }
    Ok(to_value(datetime.format(&Rfc3339).unwrap()).unwrap())
}
