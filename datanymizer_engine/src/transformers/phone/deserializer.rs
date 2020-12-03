use serde::{self, Deserialize, Deserializer};

use super::phone_format::PhoneFormat;

pub fn deserialize_phone_format<'de, D>(deserializer: D) -> Result<Option<PhoneFormat>, D::Error>
where
    D: Deserializer<'de>,
{
    let s: Option<String> = Option::deserialize(deserializer)?;
    if let Some(s) = s {
        return Ok(Some(PhoneFormat::from_format(s)));
    }
    Ok(None)
}
