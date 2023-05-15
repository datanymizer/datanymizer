use serde::Deserialize;

#[derive(Deserialize)]
pub struct EnumWrapper<T: for<'a> Deserialize<'a>>(
    #[serde(with = "serde_yaml::with::singleton_map")] pub T,
);

impl<T: for<'a> Deserialize<'a>> EnumWrapper<T> {
    pub fn parse(cfg: &str) -> Result<T, serde_yaml::Error> {
        let t: Self = serde_yaml::from_str(cfg)?;
        Ok(t.0)
    }
}
