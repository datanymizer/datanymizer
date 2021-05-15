use rand::distributions::{Distribution, Uniform};

mod base64;
pub use base64::Base64TokenTransformer;

mod base64url;
pub use base64url::Base64UrlTokenTransformer;

mod hex;
pub use hex::HexTokenTransformer;

fn rnd_chars(len: usize, src: &[char]) -> String {
    let rng = rand::thread_rng();
    let distribution = Uniform::<usize>::from(0..src.len());
    distribution
        .sample_iter(rng)
        .take(len)
        .map(|i| src[i])
        .collect::<String>()
}
