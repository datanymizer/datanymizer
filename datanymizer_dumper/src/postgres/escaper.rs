/// The escaper for values from transformers.
/// The character escaping rules for the PostgreSQL COPY command are described here:
/// https://www.postgresql.org/docs/13/sql-copy.html#id-1.9.3.55.9.2
/// If we need a NULL value in our database, we must return `\N` from the transformer.
/// Example:
/// ```yaml
/// template:
///    format: '\N'
/// ```
/// If you need the `\N` literal in your database, please return `\\N` from the transformer.
/// If you need the `\\N` literal - return `\\\N` and so on.
///
/// Warning! This behavior can be changed in the future.
pub fn replace_chars(s: &mut String) {
    if s == r#"\N"# {
        return;
    }

    let len = s.len();
    let mut new_s = None;
    let mut beginning = 0;
    let mut slash_count = 0;
    let mut position = 0;

    for (_i, c) in s.chars().enumerate() {
        position = position + c.to_string().len();

        if let Some(replacement) = match c {
            '\x08' => Some(r#"\b"#),
            '\x0C' => Some(r#"\f"#),
            '\n' => Some(r#"\n"#),
            '\r' => Some(r#"\r"#),
            '\t' => Some(r#"\t"#),
            '\x0B' => Some(r#"\v"#),
            '\\' => {
                slash_count += 1;
                Some(r#"\\"#)
            }
            _ => None,
        } {
            if new_s.is_none() {
                new_s = Some(String::with_capacity(len * 2 - position + 1));
            }
            if let Some(ref mut new_s) = new_s {
                if position > beginning {
                    new_s.push_str(&s[beginning..position - 1])
                }
                new_s.push_str(replacement);
                beginning = position;
            }
        }
    }

    if let Some(mut new_s) = new_s {
        if slash_count == len - 1 && s.ends_with('N') {
            if slash_count == 2 {
                return;
            } else {
                new_s.truncate((slash_count - 1) * 2);
            }
        }

        if beginning < len {
            new_s.push_str(&s[beginning..len])
        }
        *s = new_s;
    }
}

#[cfg(test)]
mod tests {
    use crate::postgres::escaper::replace_chars;

    #[test]
    fn replace() {
        let mut s = String::from("abc\ndef");
        replace_chars(&mut s);
        assert_eq!(s, r#"abc\ndef"#);
    }

    #[test]
    fn several() {
        let mut s = String::from("abc\r\nde\tf");
        replace_chars(&mut s);
        assert_eq!(s, r#"abc\r\nde\tf"#);
    }

    #[test]
    fn empty() {
        let mut s = String::from("");
        replace_chars(&mut s);
        assert_eq!(s, "");
    }

    #[test]
    fn at_beginning() {
        let mut s = String::from("\t123");
        replace_chars(&mut s);
        assert_eq!(s, r#"\t123"#);
    }

    #[test]
    fn at_end() {
        let mut s = String::from("abc\n");
        replace_chars(&mut s);
        assert_eq!(s, r#"abc\n"#);
    }

    #[test]
    fn slashes() {
        let mut s = String::from(r#"\ab\\c\n"#);
        replace_chars(&mut s);
        assert_eq!(s, r#"\\ab\\\\c\\n"#);
    }

    #[test]
    fn only_replacements() {
        let mut s = String::from("\r\n");
        replace_chars(&mut s);
        assert_eq!(s, r#"\r\n"#);
    }

    #[test]
    fn all_sequences() {
        let mut s = String::from("\ta\x0Bb\\c\x08\x0C\r\n");
        replace_chars(&mut s);
        assert_eq!(s, r#"\ta\vb\\c\b\f\r\n"#);
    }

    #[test]
    fn utf8_problem_case_1() {
        let mut s = String::from("Я\\");
        replace_chars(&mut s);
        assert_eq!(s, r#"Я\\"#);
    }

    #[test]
    fn utf8_problem_case_2(){
        let mut s = String::from("Яx\\");
        replace_chars(&mut s);
        assert_eq!(s, r#"Яx\\"#);
    }

    mod null_like_sequences {
        use super::*;

        #[test]
        fn one_slash() {
            let mut s = String::from(r#"\N"#);
            replace_chars(&mut s);
            assert_eq!(s, r#"\N"#);
        }

        #[test]
        fn two_slashes() {
            let mut s = String::from(r#"\\N"#);
            replace_chars(&mut s);
            assert_eq!(s, r#"\\N"#);
        }

        #[test]
        fn five_slashes() {
            let mut s = String::from(r#"\\\\\N"#);
            replace_chars(&mut s);
            assert_eq!(s, r#"\\\\\\\\N"#);
        }

        #[test]
        fn null_sequence_inside_string() {
            let mut s = String::from(r#"test\Nstring"#);
            replace_chars(&mut s);
            assert_eq!(s, r#"test\\Nstring"#);
        }
    }
}
