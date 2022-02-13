pub type GenericFloat = f64;
pub type GenericInt = isize;

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
}
