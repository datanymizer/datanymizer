/// https://docs.rs/chrono/0.3.1/chrono/format/strftime/index.html
/// https://time-rs.github.io/book/api/format-description.html
const FORMAT_REPLACEMENTS: [(&str, &str); 44] = [
    ("%Y", "[year]"),
    ("%y", "[year repr:last_two]"),
    ("%m", "[month]"),
    ("%b", "[month repr:short]"),
    ("%B", "[month repr:long]"),
    ("%h", "[month repr:short]"),
    ("%d", "[day]"),
    ("%e", "[day padding:space]"),
    ("%a", "[weekday repr:short]"),
    ("%A", "[weekday]"),
    ("%w", "[weekday repr:sunday]"),
    ("%u", "[weekday repr:monday one_indexed:true]"),
    ("%U", "[week_number repr:sunday]"),
    ("%W", "[week_number repr:monday]"),
    ("%G", "[year base:iso_week]"),
    ("%g", "[year repr:last_two base:iso_week]"),
    ("%V", "[week_number]"),
    ("%j", "[ordinal]"),
    ("%D", "[month]/[day]/[year repr:last_two]"),
    ("%x", "[day].[month].[year repr:last_two]"),
    ("%F", "[year]-[month]-[day]"),
    ("%v", "[day padding:space]-[month repr:short]-[year]"),
    ("%H", "[hour]"),
    ("%k", "[hour padding:space]"),
    ("%I", "[hour repr:12]"),
    ("%l", "[hour repr:12 padding:space]"),
    ("%P", "[period case:lower]"),
    ("%p", "[period]"),
    ("%M", "[minute]"),
    ("%S", "[second]"),
    ("%f", "[subsecond digits:9]"),
    ("%.f", ".[subsecond]"),
    ("%.3f", ".[subsecond digits:3]"),
    ("%.6f", ".[subsecond digits:6]"),
    ("%.9f", ".[subsecond digits:9]"),
    ("%R", "[hour]:[minute]"),
    ("%T", "[hour]:[minute]:[second]"),
    ("%X", "[hour]:[minute]:[second]"),
    ("%r", "[hour repr:12]:[minute]:[second] [period case:upper]"),
    ("%z", "[offset_hour sign:mandatory][offset_minute]"),
    ("%:z", "[offset_hour sign:mandatory]:[offset_minute]"),
    ("%t", "\t"),
    ("%n", "\n"),
    ("%%", "%"), // should be the last
];

pub fn convert(s: &str) -> String {
    let mut s = s.to_string();
    for (from, to) in FORMAT_REPLACEMENTS {
        s = s.replace(from, to);
    }

    s
}

#[cfg(test)]
mod tests {
    use super::*;
    use time::{format_description, macros::datetime};

    fn all_sequences() -> String {
        FORMAT_REPLACEMENTS.map(|(src, _)| src).join(" ")
    }

    #[test]
    fn replacements() {
        let all_seq = convert(all_sequences().as_str());
        assert_eq!(all_seq.find("%"), Some(all_seq.len() - 1))
    }

    #[test]
    fn format_all() {
        let all_seq = convert(all_sequences().as_str());
        let format = format_description::parse(all_seq.as_str()).unwrap();
        let dt = datetime!(1995-12-22 01:02:04.7 +5);

        assert_eq!(
            dt.format(&format).unwrap(),
            "1995 95 12 Dec December Dec 22 22 Fri Friday 6 5 51 51 1995 95 51 356 12/22/95 \
             22.12.95 1995-12-22 22-Dec-1995 01  1 01  1 am AM 02 04 700000000 .7 .700 .700000 \
             .700000000 01:02 01:02:04 01:02:04 01:02:04 AM +0500 +05:00 \t \n %"
        );
    }
}
