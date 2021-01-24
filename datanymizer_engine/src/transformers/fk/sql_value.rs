pub type GenericFloat = f32;
pub type GenericInt = isize;
pub type GenericDate = chrono::naive::NaiveDate;
pub type GenericDateTime = chrono::naive::NaiveDateTime;

pub trait AsSqlValue {
    fn sql_value(v: Self) -> String;
}

impl AsSqlValue for bool {
    fn sql_value(v: Self) -> String {
        String::from(if v { "TRUE" } else { "FALSE" })
    }
}

impl AsSqlValue for String {
    fn sql_value(v: Self) -> String {
        v
    }
}

impl AsSqlValue for Vec<String> {
    fn sql_value(v: Self) -> String {
        v.join(" ")
    }
}

impl AsSqlValue for GenericInt {
    fn sql_value(v: Self) -> String {
        v.to_string()
    }
}

impl AsSqlValue for GenericFloat {
    fn sql_value(v: Self) -> String {
        v.to_string()
    }
}

impl AsSqlValue for GenericDate {
    fn sql_value(v: Self) -> String {
        v.format("%Y-%m-%d").to_string()
    }
}

impl AsSqlValue for GenericDateTime {
    fn sql_value(v: Self) -> String {
        v.format("%Y-%m-%d %H:%M:%S").to_string()
    }
}
