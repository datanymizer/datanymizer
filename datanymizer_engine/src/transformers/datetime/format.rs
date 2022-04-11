use std::fmt::{Display, Formatter};
use time::format_description::{self, Component, FormatItem};

const PATTERN_REPLACEMENTS: [(&str, &str); 46] = [
    ("Y", "[year]"),
    ("y", "[year repr:last_two]"),
    ("m", "[month]"),
    ("b", "[month repr:short]"),
    ("B", "[month repr:long]"),
    ("h", "[month repr:short]"),
    ("d", "[day]"),
    ("e", "[day padding:space]"),
    ("a", "[weekday repr:short]"),
    ("A", "[weekday]"),
    ("w", "[weekday repr:sunday]"),
    ("u", "[weekday repr:monday one_indexed:true]"),
    ("U", "[week_number repr:sunday]"),
    ("W", "[week_number repr:monday]"),
    ("G", "[year base:iso_week]"),
    ("g", "[year repr:last_two base:iso_week]"),
    ("V", "[week_number]"),
    ("j", "[ordinal]"),
    ("D", "[month]/[day]/[year repr:last_two]"),
    ("x", "[month]/[day]/[year repr:last_two]"),
    ("F", "[year]-[month]-[day]"),
    ("v", "[day padding:space]-[month repr:short]-[year]"),
    ("H", "[hour]"),
    ("k", "[hour padding:space]"),
    ("I", "[hour repr:12]"),
    ("l", "[hour repr:12 padding:space]"),
    ("P", "[period case:lower]"),
    ("p", "[period]"),
    ("M", "[minute]"),
    ("S", "[second]"),
    ("f", "[subsecond digits:9]"),
    (".f", ".[subsecond digits:9]"),
    (".3f", ".[subsecond digits:3]"),
    (".6f", ".[subsecond digits:6]"),
    (".9f", ".[subsecond digits:9]"),
    ("R", "[hour]:[minute]"),
    ("T", "[hour]:[minute]:[second]"),
    ("X", "[hour]:[minute]:[second]"),
    ("r", "[hour repr:12]:[minute]:[second] [period case:upper]"),
    ("z", "[offset_hour sign:mandatory][offset_minute]"),
    (":z", "[offset_hour sign:mandatory]:[offset_minute]"),
    ("c", "[weekday repr:short] [month repr:short] [day padding:space] [hour]:[minute]:[second] [year]"),
    ("+", "[year]-[month]-[day]T[hour]:[minute]:[second].[subsecond digits:9][offset_hour sign:mandatory]:[offset_minute]"),
    ("t", "\t"),
    ("n", "\n"),
    ("%", "%"),
];

/// Converts formats between Chrono crate/strftime
/// (https://docs.rs/chrono/0.4.19/chrono/format/strftime/index.html)
/// and Time crate (https://time-rs.github.io/book/api/format-description.html).
///
/// Notes:
/// %C, %Z and %s are not supported (missing in the Time's format).
/// %.f works like %.9f (always 9 digits). The behaviour of the %+ pattern is the same in this regard.
/// Patterns (e.g. %x, %X, %c) are not localized (no locale support in the Time crate).
/// Modifiers "_", "-", "0" are not supported yet (you can make a feature request).
fn convert(s: &str) -> Result<String, ConvertError> {
    // 4 is just assumption
    let mut new_s = String::with_capacity(s.len() * 4);
    let mut skip_count = 0;

    for (i, c) in s.char_indices() {
        if skip_count > 0 {
            skip_count -= 1;
            continue;
        }

        if c != '%' {
            new_s.push(c);
            continue;
        }

        let substr = &s[i + 1..];
        if let Some((from, to)) = PATTERN_REPLACEMENTS
            .iter()
            .find(|(from, _)| substr.starts_with(from))
        {
            new_s.push_str(to);
            // there are only ASCII chars in the patterns, so we can use `len()` as chars' count
            skip_count = from.len();
        } else {
            return Err(ConvertError(s.to_string(), i));
        }
    }

    Ok(new_s)
}

#[derive(Debug, Eq, PartialEq, Clone)]
enum CompiledItem {
    Literal(Vec<u8>),
    Component(Component),
}

impl CompiledItem {
    fn from_format_item(item: FormatItem) -> Vec<Self> {
        match item {
            FormatItem::Literal(bs) => vec![Self::Literal(bs.to_owned())],
            FormatItem::Component(c) => vec![Self::Component(c)],
            FormatItem::Optional(i) => Self::from_format_item(i.to_owned()),
            FormatItem::Compound(nested) => Self::from_format_items(nested.to_owned()),
            FormatItem::First(nested) => nested
                .get(0)
                .map(|i| Self::from_format_item(i.to_owned()))
                .unwrap_or_default(),
            _ => vec![],
        }
    }

    fn from_format_items(items: Vec<FormatItem>) -> Vec<Self> {
        items
            .iter()
            .cloned()
            .flat_map(Self::from_format_item)
            .collect()
    }

    fn format_item(&self) -> FormatItem {
        match self {
            Self::Literal(bs) => FormatItem::Literal(bs.as_slice()),
            Self::Component(c) => FormatItem::Component(*c),
        }
    }
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct Compiled(Vec<CompiledItem>);

impl Compiled {
    pub fn compile(f: &str) -> Result<Self, CompileError> {
        let converted_format = convert(f)?;
        let format_items = format_description::parse(converted_format.as_str())?;

        Ok(Self::from_format_items(format_items))
    }

    pub fn format_items(&self) -> Vec<FormatItem> {
        self.0.iter().map(CompiledItem::format_item).collect()
    }

    fn from_format_items(fi: Vec<FormatItem>) -> Self {
        Self(CompiledItem::from_format_items(fi))
    }
}

#[derive(Debug)]
pub struct ConvertError(String, usize);

impl Display for ConvertError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "unknown pattern in the format string `{}` at {}",
            self.0, self.1
        )
    }
}

impl std::error::Error for ConvertError {}

#[derive(Debug)]
pub enum CompileError {
    Convert(ConvertError),
    Format(time::error::InvalidFormatDescription),
}

impl Display for CompileError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

impl std::error::Error for CompileError {}

impl From<ConvertError> for CompileError {
    fn from(err: ConvertError) -> Self {
        Self::Convert(err)
    }
}

impl From<time::error::InvalidFormatDescription> for CompileError {
    fn from(err: time::error::InvalidFormatDescription) -> Self {
        Self::Format(err)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use time::{macros::datetime, OffsetDateTime};

    mod convert {
        use super::*;

        #[test]
        fn replacements() {
            let all = PATTERN_REPLACEMENTS
                .map(|(src, _)| format!("%{}", src))
                .join(" ");
            let all = convert(all.as_str()).unwrap();

            assert_eq!(all.find("%"), Some(all.len() - 1));
        }
    }

    mod strftime {
        use super::*;

        fn strftime(dt: &OffsetDateTime, f: &str) -> String {
            let f = Compiled::compile(f).unwrap();
            dt.format(&f.format_items()).unwrap()
        }

        #[test]
        fn ymd_patterns() {
            let f = "%Y-%m-%d %y %B %b %h %e %D %x %F %v %j";
            let dt = datetime!(2010-02-04 00:00:00 UTC);
            assert_eq!(
                strftime(&dt, f),
                "2010-02-04 10 February Feb Feb  4 02/04/10 02/04/10 2010-02-04  4-Feb-2010 035"
            );
        }

        #[test]
        fn week_day_patterns() {
            let f = "%a %A %w %u";
            let dt = datetime!(2010-02-04 00:00:00 UTC);
            assert_eq!(strftime(&dt, f), "Thu Thursday 5 4");
        }

        #[test]
        fn week_number_patterns() {
            let f = "%U %W %V %G %g";

            let dt = datetime!(2018-01-06 00:00:00 UTC);
            assert_eq!(strftime(&dt, f), "00 01 01 2018 18");

            let dt = datetime!(2018-01-07 00:00:00 UTC);
            assert_eq!(strftime(&dt, f), "01 01 01 2018 18");

            let dt = datetime!(2018-01-08 00:00:00 UTC);
            assert_eq!(strftime(&dt, f), "01 02 02 2018 18");

            let dt = datetime!(2017-01-01 00:00:00 UTC);
            assert_eq!(strftime(&dt, f), "01 00 52 2016 16");

            let dt = datetime!(2017-01-02 00:00:00 UTC);
            assert_eq!(strftime(&dt, f), "01 01 01 2017 17");
        }

        #[test]
        fn hms_patterns() {
            let f = "%H %k %I %l %P %p %M %S %R %T %X %r";

            let dt = datetime!(2018-01-06 01:02:04 UTC);
            assert_eq!(
                strftime(&dt, f),
                "01  1 01  1 am AM 02 04 01:02 01:02:04 01:02:04 01:02:04 AM"
            );

            let dt = datetime!(2018-01-06 13:32:34 UTC);
            assert_eq!(
                strftime(&dt, f),
                "13 13 01  1 pm PM 32 34 13:32 13:32:34 13:32:34 01:32:34 PM"
            );
        }

        #[test]
        fn tz_patterns() {
            let f = "%z %:z";

            let dt = datetime!(2018-01-06 01:02:04 +5);
            assert_eq!(strftime(&dt, f), "+0500 +05:00");

            let dt = datetime!(2018-01-06 01:02:04 -1:30);
            assert_eq!(strftime(&dt, f), "-0130 -01:30");
        }

        #[test]
        fn subsec_patterns() {
            let f = "%f %.f %.3f %.6f %.9f";
            let dt = datetime!(2018-01-06 01:02:04.01234567 UTC);
            assert_eq!(
                strftime(&dt, f),
                "012345670 .012345670 .012 .012345 .012345670"
            );
        }

        #[test]
        fn full_patterns() {
            let f = "%c %+";
            let dt = datetime!(2018-01-06 01:02:04.5 -2:00);
            assert_eq!(
                strftime(&dt, f),
                "Sat Jan  6 01:02:04 2018 2018-01-06T01:02:04.500000000-02:00"
            );
        }

        #[test]
        fn escape_symbols() {
            let f = "%t%n%%d";
            let dt = OffsetDateTime::now_utc();
            assert_eq!(strftime(&dt, f), "\t\n%d");
        }

        #[test]
        fn unicode() {
            let dt = datetime!(1995-12-22 00:00:00 +5);
            assert_eq!(
                strftime(&dt, "Год: %Y, месяц: %m, день: %d"),
                "Год: 1995, месяц: 12, день: 22"
            );
        }

        #[test]
        fn last_percent() {
            let err = convert("%Y-%m-%").err().unwrap();
            assert_eq!(
                err.to_string(),
                "unknown pattern in the format string `%Y-%m-%` at 6"
            );
        }

        #[test]
        fn unknown_pattern() {
            let err = convert("%y-%@-%d").err().unwrap();
            assert_eq!(
                err.to_string(),
                "unknown pattern in the format string `%y-%@-%d` at 3"
            );
        }
    }

    mod from_format_item {
        use super::*;
        use time::format_description::modifier::{Day, Month, Year};

        #[test]
        fn literal() {
            let s = "abc";
            let fi = FormatItem::Literal(s.as_bytes());

            let dt = datetime!(2018-05-06 00:00:00 UTC);
            assert_eq!(dt.format(&fi).unwrap(), "abc");

            assert_eq!(
                CompiledItem::from_format_item(fi),
                vec![CompiledItem::Literal(s.as_bytes().to_vec())]
            );
        }

        #[test]
        fn component() {
            let year = Component::Year(Year::default());
            let fi = FormatItem::Component(year);

            let dt = datetime!(2018-05-06 00:00:00 UTC);
            assert_eq!(dt.format(&fi).unwrap(), "2018");

            assert_eq!(
                CompiledItem::from_format_item(fi),
                vec![CompiledItem::Component(year)]
            )
        }

        #[test]
        fn optional() {
            let s = "abc";
            let child = FormatItem::Literal(s.as_bytes());
            let fi = FormatItem::Optional(&child);

            let dt = datetime!(2018-05-06 00:00:00 UTC);
            assert_eq!(dt.format(&fi).unwrap(), "abc");

            assert_eq!(
                CompiledItem::from_format_item(fi),
                vec![CompiledItem::Literal(s.as_bytes().to_vec())]
            )
        }

        #[test]
        fn compound() {
            let s = "abc";
            let month = Component::Month(Month::default());

            let children = vec![
                FormatItem::Literal(s.as_bytes()),
                FormatItem::Component(month),
            ];

            let fi = FormatItem::Compound(children.as_slice());

            let dt = datetime!(2018-05-06 00:00:00 UTC);
            assert_eq!(dt.format(&fi).unwrap(), "abc05");

            assert_eq!(
                CompiledItem::from_format_item(fi),
                vec![
                    CompiledItem::Literal(s.as_bytes().to_vec()),
                    CompiledItem::Component(month),
                ]
            )
        }

        #[test]
        fn first() {
            let s = "abc";
            let day = Component::Day(Day::default());

            let children = vec![
                FormatItem::Component(day),
                FormatItem::Literal(s.as_bytes()),
            ];

            let fi = FormatItem::First(children.as_slice());

            let dt = datetime!(2018-01-06 00:00:00 UTC);
            assert_eq!(dt.format(&fi).unwrap(), "06");

            assert_eq!(
                CompiledItem::from_format_item(fi),
                vec![CompiledItem::Component(day)]
            );
        }
    }
}
