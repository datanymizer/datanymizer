// https://www.postgresql.org/docs/13/sql-copy.html#id-1.9.3.55.9.2
pub fn replace_chars(s: &mut String) {
    let len = s.len();
    let mut new_s = None;
    let mut beginning = 0;

    for (i, c) in s.chars().enumerate() {
        if let Some(replacement) = match c {
            '\x08' => Some(r#"\b"#),
            '\x0C' => Some(r#"\f"#),
            '\n' => Some(r#"\n"#),
            '\r' => Some(r#"\r"#),
            '\t' => Some(r#"\t"#),
            '\x0B' => Some(r#"\v"#),
            '\\' => Some(r#"\\"#),
            _ => None,
        } {
            if new_s.is_none() {
                new_s = Some(String::with_capacity(len * 2 - i));
            }
            if let Some(ref mut new_s) = new_s {
                if i > beginning {
                    new_s.push_str(&s[beginning..i])
                }
                new_s.push_str(replacement);
                beginning = i + 1;
            }
        }
    }

    if let Some(mut new_s) = new_s {
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
        assert_eq!(s, r#"abc\ndef"#)
    }

    #[test]
    fn several() {
        let mut s = String::from("abc\r\nde\tf");
        replace_chars(&mut s);
        assert_eq!(s, r#"abc\r\nde\tf"#)
    }

    #[test]
    fn empty() {
        let mut s = String::from("");
        replace_chars(&mut s);
        assert_eq!(s, "")
    }

    #[test]
    fn at_beginning() {
        let mut s = String::from("\t123");
        replace_chars(&mut s);
        assert_eq!(s, r#"\t123"#)
    }

    #[test]
    fn at_end() {
        let mut s = String::from("abc\n");
        replace_chars(&mut s);
        assert_eq!(s, r#"abc\n"#)
    }

    #[test]
    fn slashes() {
        let mut s = String::from(r#"\ab\\c\n"#);
        replace_chars(&mut s);
        assert_eq!(s, r#"\\ab\\\\c\\n"#)
    }

    #[test]
    fn only_replacements() {
        let mut s = String::from("\r\n");
        replace_chars(&mut s);
        assert_eq!(s, r#"\r\n"#)
    }

    #[test]
    fn all_sequences() {
        let mut s = String::from("\ta\x0Bb\\c\x08\x0C\r\n");
        replace_chars(&mut s);
        assert_eq!(s, r#"\ta\vb\\c\b\f\r\n"#)
    }
}
