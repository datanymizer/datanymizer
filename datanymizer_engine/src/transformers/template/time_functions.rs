use crate::utils::format_time::Compiled;
use serde_json::value::{from_value, to_value};
use std::collections::HashMap;
use tera::{try_get_value, Error, Result, Tera, Value};
use time::{
    format_description::{self, well_known::Rfc3339},
    Date, OffsetDateTime,
};
use time_tz::{timezones, OffsetDateTimeExt};

// We reimplement the `now()` function and the `date()` filter for Tera without using `chrono`
// (because of a security vulnerability in the latter).

pub fn register(t: &mut Tera) {
    t.register_function("now", now);
    t.register_filter("date", date);
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

pub fn date(value: &Value, args: &HashMap<String, Value>) -> Result<Value> {
    let format = match args.get("format") {
        Some(val) => try_get_value!("date", "format", String, val),
        None => "%Y-%m-%d".to_string(),
    };

    let compiled = Compiled::compile(format.as_str());
    if compiled.is_err() {
        return Err(Error::msg(format!("Invalid date format `{}`", format)));
    }
    let compiled = compiled.unwrap();

    let timezone = match args.get("timezone") {
        Some(val) => {
            let timezone = try_get_value!("date", "timezone", String, val);
            match timezones::get_by_name(timezone.as_str()) {
                Some(timezone) => Some(timezone),
                None => {
                    return Err(Error::msg(format!(
                        "Error parsing `{}` as a timezone",
                        timezone
                    )));
                }
            }
        }
        None => None,
    };

    let formatted = match value {
        Value::Number(n) => match n.as_i64() {
            Some(i) => {
                let datetime = OffsetDateTime::from_unix_timestamp(i);
                if let Ok(datetime) = datetime {
                    datetime.format(&compiled.format_items())
                } else {
                    return Err(Error::msg(format!("Invalid timestamp: {}", i)));
                }
            }
            None => {
                return Err(Error::msg(format!(
                    "Filter `date` was invoked on a float: {}",
                    n
                )))
            }
        },
        Value::String(s) => {
            if s.contains('T') {
                match OffsetDateTime::parse(s, &Rfc3339) {
                    Ok(val) => match timezone {
                        Some(timezone) => {
                            val.to_timezone(timezone).format(&compiled.format_items())
                        }
                        None => val.format(&compiled.format_items()),
                    },
                    // Like NaiveDateTime in the original
                    Err(_) => match OffsetDateTime::parse(format!("{}Z", s).as_str(), &Rfc3339) {
                        Ok(val) => val.format(&compiled.format_items()),
                        Err(_) => {
                            return Err(Error::msg(format!(
                                "Error parsing `{:?}` as rfc3339 date or naive datetime",
                                s
                            )));
                        }
                    },
                }
            } else {
                let format = format_description::parse("[year]-[month]-[day]").unwrap();
                match Date::parse(s, &format) {
                    Ok(val) => val
                        .with_hms(0, 0, 0)
                        .unwrap()
                        .assume_utc()
                        .format(&compiled.format_items()),
                    Err(_) => {
                        return Err(Error::msg(format!(
                            "Error parsing `{:?}` as YYYY-MM-DD date",
                            s
                        )));
                    }
                }
            }
        }
        _ => {
            return Err(Error::msg(format!(
                "Filter `date` received an incorrect type for arg `value`: \
                 got `{:?}` but expected i64|u64|String",
                value
            )));
        }
    };

    match formatted {
        Ok(formatted) => to_value(formatted).map_err(Error::json),
        Err(e) => Err(Error::msg(format!("Formatting error: {}", e))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod now {
        use super::*;

        #[test]
        fn now_default() {
            let args = HashMap::new();

            let res = now(&args).unwrap();
            assert!(res.is_string());
            assert!(res.as_str().unwrap().contains("T"));
        }

        #[test]
        fn now_datetime_utc() {
            let mut args = HashMap::new();
            args.insert("utc".to_string(), to_value(true).unwrap());

            let res = now(&args).unwrap();
            assert!(res.is_string());
            let val = res.as_str().unwrap();
            println!("{}", val);
            assert!(val.contains("T"));
            assert!(val.contains("Z"));
        }

        #[test]
        fn now_timestamp() {
            let mut args = HashMap::new();
            args.insert("timestamp".to_string(), to_value(true).unwrap());

            let res = now(&args).unwrap();
            assert!(res.is_number());
        }
    }

    mod date {
        use super::*;

        #[test]
        fn date_default() {
            let args = HashMap::new();
            let result = date(&to_value(1482720453).unwrap(), &args);
            assert!(result.is_ok());
            assert_eq!(result.unwrap(), to_value("2016-12-26").unwrap());
        }

        #[test]
        fn date_custom_format() {
            let mut args = HashMap::new();
            args.insert("format".to_string(), to_value("%Y-%m-%d %H:%M").unwrap());
            let result = date(&to_value(1482720453).unwrap(), &args);
            assert!(result.is_ok());
            assert_eq!(result.unwrap(), to_value("2016-12-26 02:47").unwrap());
        }

        #[test]
        fn date_errors_on_incorrect_format() {
            let mut args = HashMap::new();
            args.insert("format".to_string(), to_value("%2f").unwrap());
            let result = date(&to_value(1482720453).unwrap(), &args);
            assert!(result.is_err());
        }

        #[test]
        fn date_rfc3339() {
            let args = HashMap::new();
            let result = date(&to_value("2021-10-07").unwrap(), &args);
            assert!(result.is_ok());
            assert_eq!(result.unwrap(), to_value("2021-10-07").unwrap());
        }

        #[test]
        fn date_rfc3339_preserves_timezone() {
            let mut args = HashMap::new();
            args.insert("format".to_string(), to_value("%Y-%m-%d %z").unwrap());
            let result = date(&to_value("1996-12-19T16:39:57-08:00").unwrap(), &args);
            assert!(result.is_ok());
            assert_eq!(result.unwrap(), to_value("1996-12-19 -0800").unwrap());
        }

        #[test]
        fn date_yyyy_mm_dd() {
            let mut args = HashMap::new();
            args.insert(
                "format".to_string(),
                to_value("%a, %d %b %Y %H:%M:%S %z").unwrap(),
            );
            let result = date(&to_value("2017-03-05").unwrap(), &args);
            assert!(result.is_ok());
            assert_eq!(
                result.unwrap(),
                to_value("Sun, 05 Mar 2017 00:00:00 +0000").unwrap()
            );
        }

        #[test]
        fn date_from_naive_datetime() {
            let mut args = HashMap::new();
            args.insert(
                "format".to_string(),
                to_value("%a, %d %b %Y %H:%M:%S").unwrap(),
            );
            let result = date(&to_value("2017-03-05T00:00:00.602").unwrap(), &args);
            println!("{:?}", result);
            assert!(result.is_ok());
            assert_eq!(
                result.unwrap(),
                to_value("Sun, 05 Mar 2017 00:00:00").unwrap()
            );
        }

        #[test]
        fn date_format_doesnt_panic() {
            let mut args = HashMap::new();
            args.insert("format".to_string(), to_value("%+S").unwrap());
            let result = date(&to_value("2017-01-01T00:00:00").unwrap(), &args);
            assert!(result.is_ok());
        }

        #[test]
        fn date_with_timezone() {
            let mut args = HashMap::new();
            args.insert(
                "timezone".to_string(),
                to_value("America/New_York").unwrap(),
            );
            let result = date(&to_value("2019-09-19T01:48:44.581Z").unwrap(), &args);
            assert!(result.is_ok());
            assert_eq!(result.unwrap(), to_value("2019-09-18").unwrap());
        }

        #[test]
        fn date_with_invalid_timezone() {
            let mut args = HashMap::new();
            args.insert("timezone".to_string(), to_value("Narnia").unwrap());
            let result = date(&to_value("2019-09-19T01:48:44.581Z").unwrap(), &args);
            assert!(result.is_err());
            assert_eq!(
                result.err().unwrap().to_string(),
                "Error parsing `Narnia` as a timezone"
            );
        }
    }
}
