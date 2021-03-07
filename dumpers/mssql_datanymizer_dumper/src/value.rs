use chrono::{DateTime, NaiveDate, NaiveDateTime, NaiveTime, SecondsFormat, Utc};
use std::fmt::{self, Display};
use tiberius::{ColumnData, FromSqlOwned};

#[derive(Default)]
pub(crate) struct Value {
    pub str: String,
    pub format: DumpFormat,
}

impl Value {
    fn new<S: ToString>(str: S, format: DumpFormat) -> Self {
        Self {
            str: str.to_string(),
            format,
        }
    }

    pub fn dump_string<S: ToString>(f: DumpFormat, s: S) -> String {
        match f {
            DumpFormat::Raw => s.to_string(),
            DumpFormat::Quote => format!("N'{}'", s.to_string().replace('\'', "''")),
            DumpFormat::Null => "NULL".to_string(),
        }
    }

    pub fn into_dump_string(self) -> String {
        Self::dump_string(self.format, self.str)
    }

    fn from_option<S: ToString + Display>(v: Option<S>, f: DumpFormat) -> Self {
        v.as_ref().map_or(Self::default(), |s| Self::new(s, f))
    }
}

impl<'a> From<ColumnData<'a>> for Value {
    fn from(d: ColumnData<'_>) -> Self {
        match d {
            ColumnData::U8(v) => Self::from_option(v, DumpFormat::Raw),
            ColumnData::I16(v) => Self::from_option(v, DumpFormat::Raw),
            ColumnData::I32(v) => Self::from_option(v, DumpFormat::Raw),
            ColumnData::I64(v) => Self::from_option(v, DumpFormat::Raw),
            ColumnData::F32(v) => Self::from_option(v, DumpFormat::Raw),
            ColumnData::F64(v) => Self::from_option(v, DumpFormat::Raw),
            ColumnData::Bit(v) => {
                Self::from_option(v.map(|v| if v { "1" } else { "0" }), DumpFormat::Raw)
            }
            ColumnData::String(v) => Self::from_option(v, DumpFormat::Quote),
            ColumnData::Guid(v) => Self::from_option(v, DumpFormat::Quote),
            ColumnData::Binary(v) => Self::from_option(
                v.map(|data| {
                    let mut s = String::with_capacity(data.len() * 2 + 2);
                    s.push_str("0x");
                    for byte in data.as_ref() {
                        s.push_str(format!("{:02x}", byte).as_str());
                    }

                    s
                }),
                DumpFormat::Raw,
            ),
            ColumnData::Numeric(v) => Self::from_option(v, DumpFormat::Raw),
            ColumnData::Xml(v) => {
                Self::from_option(v.map(|xml| xml.to_string()), DumpFormat::Quote)
            }
            ColumnData::DateTime(v) => Self::from_option(
                NaiveDateTime::from_sql_owned(ColumnData::DateTime(v))
                    .expect("invalid sql date")
                    .map(|date| date.format("%Y-%m-%dT%H:%M:%S%.3f")),
                DumpFormat::Quote,
            ),
            ColumnData::SmallDateTime(v) => Self::from_option(
                NaiveDateTime::from_sql_owned(ColumnData::SmallDateTime(v))
                    .expect("invalid sql date")
                    .map(|date| date.format("%Y-%m-%dT%H:%M:00")),
                DumpFormat::Quote,
            ),
            ColumnData::Time(v) => Self::from_option(
                NaiveTime::from_sql_owned(ColumnData::Time(v))
                    .expect("invalid sql date")
                    .map(|date| date.format("%H:%M:%S%.9f")),
                DumpFormat::Quote,
            ),
            ColumnData::Date(v) => Self::from_option(
                NaiveDate::from_sql_owned(ColumnData::Date(v))
                    .expect("invalid sql date")
                    .map(|date| date.format("%Y-%m-%d")),
                DumpFormat::Quote,
            ),
            ColumnData::DateTime2(v) => Self::from_option(
                NaiveDateTime::from_sql_owned(ColumnData::DateTime2(v))
                    .expect("invalid sql date")
                    .map(|date| date.format("%Y-%m-%dT%H:%M:%S%.9f")),
                DumpFormat::Quote,
            ),
            ColumnData::DateTimeOffset(v) => Self::from_option(
                // TODO: Check TZ
                DateTime::<Utc>::from_sql_owned(ColumnData::DateTimeOffset(v))
                    .expect("invalid sql date")
                    .map(|date| date.to_rfc3339_opts(SecondsFormat::Nanos, false)),
                DumpFormat::Quote,
            ),
        }
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.str)
    }
}

#[derive(Copy, Clone)]
pub enum DumpFormat {
    Raw,
    Quote,
    Null,
}

impl Default for DumpFormat {
    fn default() -> Self {
        Self::Null
    }
}
