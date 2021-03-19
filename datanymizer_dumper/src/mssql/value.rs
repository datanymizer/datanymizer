use std::fmt::{self, Display};
use tiberius::{error::Error, ColumnData, FromSql};

pub(crate) struct Value(String);

impl<'a> FromSql<'a> for Value {
    fn from_sql(value: &'a ColumnData<'static>) -> Result<Option<Self>, Error> {
        Ok(Some(Self(match value {
            ColumnData::U8(v) => Self::value_to_string(v),
            ColumnData::I16(v) => Self::value_to_string(v),
            ColumnData::I32(v) => Self::value_to_string(v),
            ColumnData::I64(v) => Self::value_to_string(v),
            ColumnData::F32(v) => Self::value_to_string(v),
            ColumnData::F64(v) => Self::value_to_string(v),
            ColumnData::Bit(v) => Self::value_to_string(v),
            ColumnData::String(v) => Self::value_to_string(v),
            ColumnData::Guid(v) => Self::value_to_string(v),
            // ColumnData::Binary(v) => Self::value_to_string(v),
            // ColumnData::Numeric(v) => Self::value_to_string(v),
            // ColumnData::Xml(v) => Self::value_to_string(v),
            // ColumnData::DateTime(v) => Self::value_to_string(v),
            // ColumnData::SmallDateTime(v) => Self::value_to_string(v),
            // ColumnData::Time(v) => Self::value_to_string(v),
            // ColumnData::Date(v) => Self::value_to_string(v),
            // ColumnData::DateTime2(v) => Self::value_to_string(v),
            // ColumnData::DateTimeOffset(v) => Self::value_to_string(v),
            _ => panic!("unsupported database type"),
        })))
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Value {
    fn value_to_string<S: ToString>(v: &Option<S>) -> String {
        v.as_ref().map_or(String::new(), |n| n.to_string())
    }
}
