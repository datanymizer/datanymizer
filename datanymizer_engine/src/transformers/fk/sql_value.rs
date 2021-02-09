pub type GenericFloat = f64;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bool() {
        assert_eq!(bool::sql_value(true), "TRUE");
        assert_eq!(bool::sql_value(false), "FALSE");
    }

    #[test]
    fn string() {
        assert_eq!(String::sql_value(String::from("str")), "str");
    }

    #[test]
    fn vec_of_string() {
        assert_eq!(
            Vec::<String>::sql_value(vec![String::from("str1"), String::from("str2")]),
            "str1 str2"
        );
    }

    #[test]
    fn generic_int() {
        assert_eq!(GenericInt::sql_value(12), "12");
    }

    #[test]
    fn generic_float() {
        assert_eq!(GenericFloat::sql_value(17.01), "17.01");
    }

    #[test]
    fn generic_date() {
        assert_eq!(
            GenericDate::sql_value(GenericDate::from_ymd(2012, 3, 4)),
            "2012-03-04"
        );
    }

    #[test]
    fn generic_datetime() {
        let dt = GenericDateTime::new(
            GenericDate::from_ymd(2020, 5, 20),
            chrono::naive::NaiveTime::from_hms(9, 12, 1),
        );
        assert_eq!(GenericDateTime::sql_value(dt), "2020-05-20 09:12:01");
    }
}
