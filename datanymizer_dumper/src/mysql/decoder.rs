use std::fmt::Display;
use anyhow::anyhow;
use sqlx::{mysql::{MySqlTypeInfo, MySqlValueRef}, Decode, MySql, TypeInfo};
use time::{Date, OffsetDateTime, Time};

pub enum Decoder {
    Str,
    Unsigned,
    Signed,
    Float,
    Bool,
    Date,
    Datetime,
    Time,
    Null,
}

impl Decoder {
    pub fn decode(
        &self,
        value: MySqlValueRef<'_>,
    ) -> anyhow::Result<String, Box<dyn std::error::Error + 'static + Send + Sync>> {
        match self {
            Decoder::Str => <&str as Decode<MySql>>::decode(value).map(|v| v.to_string()),
            Decoder::Unsigned => <u64 as Decode<MySql>>::decode(value).map(|v| v.to_string()),
            Decoder::Signed => <i64 as Decode<MySql>>::decode(value).map(|v| v.to_string()),
            Decoder::Float => <f64 as Decode<MySql>>::decode(value).map(|v| v.to_string()),
            Decoder::Bool => <bool as Decode<MySql>>::decode(value).map(|v| v.to_string()),
            Decoder::Date => <Date as Decode<MySql>>::decode(value).map(|v| v.to_string()),
            Decoder::Datetime => {
                <OffsetDateTime as Decode<MySql>>::decode(value).map(|v| v.to_string())
            }
            Decoder::Time => <Time as Decode<MySql>>::decode(value).map(|v| v.to_string()),
            Decoder::Null => Ok("NULL".into()),
        }
    }

    pub fn decode_and_quote(
        &self,
        value: MySqlValueRef<'_>,
    ) -> anyhow::Result<String, Box<dyn std::error::Error + 'static + Send + Sync>> {
        self.decode(value)
            .map(|v| self.prepare_output(&v).unwrap_or(v))
    }

    pub fn prepare_output<T: Display>(&self, s: T) -> Option<String> {
        match self {
            Decoder::Str => Some(quote_and_escape(s)),
            Decoder::Date | Decoder::Datetime | Decoder::Time => Some(quote(s)),
            _ => None,
        }
    }
}

impl TryFrom<&MySqlTypeInfo> for Decoder {
    type Error = anyhow::Error;

    fn try_from(value: &MySqlTypeInfo) -> anyhow::Result<Self, Self::Error> {
        match value.name() {
            "BOOLEAN" => Ok(Decoder::Bool),
            "TINYINT UNSIGNED" | "SMALLINT UNSIGNED" | "INT UNSIGNED" | "MEDIUMINT UNSIGNED"
            | "BIGINT UNSIGNED" | "YEAR" => Ok(Decoder::Unsigned),
            "TINYINT" | "SMALLINT" | "INT" | "MEDIUMINT" | "BIGINT" => Ok(Decoder::Signed),
            "FLOAT" | "DOUBLE" => Ok(Decoder::Float),
            "NULL" => Ok(Decoder::Null),
            "DATE" => Ok(Decoder::Date),
            "TIMESTAMP" | "DATETIME" => Ok(Decoder::Datetime),
            "TIME" => Ok(Decoder::Time),
            "VARCHAR" | "ENUM" | "CHAR" | "TINYTEXT" | "TEXT" | "MEDIUMTEXT" | "LONGTEXT" => {
                Ok(Decoder::Str)
            }
            // "BIT",
            // "SET"
            // "DECIMAL"
            // "GEOMETRY"
            // "JSON"
            // "BINARY"
            // "VARBINARY"
            // "TINYBLOB" | "BLOB" | "MEDIUMBLOB" | "LONGBLOB"
            _ => Err(anyhow!("Unsupported DB type")),
        }
    }
}

fn quote_and_escape<T: Display>(src: T) -> String {
    let mut s = format!("'{}'", src);
    let len = s.len();
    let mut new_s = None;
    let mut beginning = 0;

    for (i, c) in s.char_indices() {
        if i == 0 || i == len - 1 {
            continue;
        }
        if let Some(replacement) = match c {
            '\\' => Some(r#"\\"#),
            '\'' => Some(r#"\'"#),
            _ => None,
        } {
            if new_s.is_none() {
                new_s = Some(String::with_capacity(len * 2 - i));
            }
            if let Some(ref mut new_s) = new_s {
                if i > beginning {
                    new_s.push_str(&s[beginning..i])
                }
                new_s.push_str(replacement);
                beginning = i + 1;
            }
        }
    }

    if let Some(mut new_s) = new_s {
        if beginning < len {
            new_s.push_str(&s[beginning..len])
        }
        s = new_s;
    }
    s
}

fn quote<T: Display>(src: T) -> String {
    format!("'{}'", src)
}
