use rand::distributions::{Distribution, Uniform};

pub fn rnd_chars(len: usize, src: &[char]) -> String {
    let rng = rand::thread_rng();
    let distribution = Uniform::<usize>::from(0..src.len());
    distribution
        .sample_iter(rng)
        .take(len)
        .map(|i| src[i])
        .collect::<String>()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_char() {
        let chars = vec!['a'];

        assert_eq!(rnd_chars(0, &chars), "");
        assert_eq!(rnd_chars(1, &chars), "a");
        assert_eq!(rnd_chars(4, &chars), "aaaa");
    }

    #[test]
    fn different_chars() {
        let chars = vec!['a', 'b', 'c'];

        assert_eq!(rnd_chars(0, &chars), "");

        let s = rnd_chars(1, &chars);
        for ch in s.chars() {
            assert!(chars.contains(&ch));
        }

        let s = rnd_chars(5, &chars);
        for ch in s.chars() {
            assert!(chars.contains(&ch));
        }
    }
}
