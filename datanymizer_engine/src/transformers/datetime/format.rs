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

pub fn convert2(s: &str) -> String {
    let mut new_s = s.to_string();
    let mut i = 0;

    loop {
        if i >= new_s.len() {
            break;
        }

        let inc = if new_s[i..].starts_with('%') {
            if let Some((from, to)) = PATTERN_REPLACEMENTS
                .iter()
                .find(|(from, _)| new_s[i + 1..].starts_with(from))
            {
                new_s.replace_range(i..=i + from.len(), to);
                to.len()
            } else {
                panic!("Unexpected pattern in the format string `{}` at {}", s, i);
            }
        } else {
            1
        };

        i += inc;
    }

    new_s
}

pub fn convert(s: &str) -> String {
    // 4 is just assumption
    let mut new_s = String::with_capacity(s.len() * 4);
    let mut skip_count = 0;

    for (i, c) in s.char_indices() {
        if skip_count > 0 {
            skip_count -= 1;
            continue;
        }

        if c == '%' {
            if let Some((from, to)) = PATTERN_REPLACEMENTS
                .iter()
                .find(|(from, _)| s[i + 1..].starts_with(from))
            {
                new_s.push_str(to);
                // there are only ASCII chars in the patterns, so we can use `len()` as chars' count
                skip_count = from.len();
            } else {
                panic!("Unexpected pattern in the format string `{}` at {}", s, i);
            }
        } else {
            new_s.push(c);
        }
    }

    new_s
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
        let f = convert(f);
        let f = format_description::parse(f.as_str()).unwrap();
        dt.format(&f).unwrap()
    }

    #[test]
    fn replacements() {
        let all = convert(all_patterns().as_str());
        assert_eq!(all.find("%"), Some(all.len() - 1))
    }

    #[test]
    fn format_all() {
        let dt = datetime!(1995-12-22 01:02:04.7 +5);

        assert_eq!(
            strftime(&dt, all_patterns().as_str()),
            "1995 95 12 Dec December Dec 22 22 Fri Friday 6 5 51 51 1995 95 51 356 12/22/95 \
             22.12.95 1995-12-22 22-Dec-1995 01  1 01  1 am AM 02 04 700000000 .7 .700 .700000 \
             .700000000 01:02 01:02:04 01:02:04 01:02:04 AM +0500 +05:00 \t \n %"
        );
    }

    #[test]
    fn escape_percent() {
        let dt = datetime!(1995-12-22 00:00:00 +5);
        assert_eq!(strftime(&dt, "%%d"), "%d");
    }
}
