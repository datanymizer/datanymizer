use std::fmt::{Display, Formatter};

/// https://docs.rs/chrono/0.3.1/chrono/format/strftime/index.html
/// https://time-rs.github.io/book/api/format-description.html
const PATTERN_REPLACEMENTS: [(&str, &str); 44] = [
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
    ("x", "[day].[month].[year repr:last_two]"),
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
    (".f", ".[subsecond]"),
    (".3f", ".[subsecond digits:3]"),
    (".6f", ".[subsecond digits:6]"),
    (".9f", ".[subsecond digits:9]"),
    ("R", "[hour]:[minute]"),
    ("T", "[hour]:[minute]:[second]"),
    ("X", "[hour]:[minute]:[second]"),
    ("r", "[hour repr:12]:[minute]:[second] [period case:upper]"),
    ("z", "[offset_hour sign:mandatory][offset_minute]"),
    (":z", "[offset_hour sign:mandatory]:[offset_minute]"),
    ("t", "\t"),
    ("n", "\n"),
    ("%", "%"),
];

#[derive(Debug)]
pub enum ConvertError {
    UnexpectedPattern(String, usize),
    UnexpectedEnd(String),
}

impl Display for ConvertError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnexpectedPattern(s, i) => write!(
                f,
                "unexpected pattern in the format string `{}` at {}",
                s, i
            ),
            Self::UnexpectedEnd(s) => write!(f, "unexpected end of format string `{}`", s),
        }
    }
}

impl std::error::Error for ConvertError {}

pub fn convert(s: &str) -> Result<String, ConvertError> {
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

        if let Some(substr) = s.get(i + 1..) {
            if let Some((from, to)) = PATTERN_REPLACEMENTS
                .iter()
                .find(|(from, _)| substr.starts_with(from))
            {
                new_s.push_str(to);
                // there are only ASCII chars in the patterns, so we can use `len()` as chars' count
                skip_count = from.len();
            } else {
                return Err(ConvertError::UnexpectedPattern(s.to_string(), i));
            }
        } else {
            return Err(ConvertError::UnexpectedEnd(s.to_string()));
        }
    }

    Ok(new_s)
}

#[cfg(test)]
mod tests {
    use super::*;
    use time::{format_description, macros::datetime, OffsetDateTime};

    fn all_patterns() -> String {
        PATTERN_REPLACEMENTS
            .map(|(src, _)| format!("%{}", src))
            .join(" ")
    }

    fn strftime(dt: &OffsetDateTime, f: &str) -> String {
        let f = convert(f).unwrap();
        let f = format_description::parse(f.as_str()).unwrap();
        dt.format(&f).unwrap()
    }

    #[test]
    fn replacements() {
        let all = convert(all_patterns().as_str()).unwrap();
        assert_eq!(all.find("%"), Some(all.len() - 1));
    }

    #[test]
    fn check_all_patterns() {
        let dt = datetime!(2010-02-04 01:02:04.7 +5);

        assert_eq!(
            strftime(&dt, all_patterns().as_str()),
            "2010 10 02 Feb February Feb 04  4 Thu Thursday 5 4 05 05 2010 10 05 035 02/04/10 \
             04.02.10 2010-02-04  4-Feb-2010 01  1 01  1 am AM 02 04 700000000 .7 .700 .700000 \
             .700000000 01:02 01:02:04 01:02:04 01:02:04 AM +0500 +05:00 \t \n %"
        );
    }

    #[test]
    fn escape_percent() {
        let dt = OffsetDateTime::now_utc();
        assert_eq!(strftime(&dt, "%%d"), "%d");
    }

    #[test]
    fn unicode() {
        let dt = datetime!(1995-12-22 00:00:00 +5);
        assert_eq!(
            strftime(&dt, "Год: %Y, месяц: %m, день: %d"),
            "Год: 1995, месяц: 12, день: 22"
        );
    }
}
