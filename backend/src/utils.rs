use serde::{Deserialize, Deserializer};

pub fn trimmed_string<'de, D: Deserializer<'de>>(d: D) -> Result<String, D::Error> {
    let s = String::deserialize(d)?;
    Ok(s.trim().to_string())
}

pub fn trimmed_option<'de, D: Deserializer<'de>>(d: D) -> Result<Option<String>, D::Error> {
    let opt = Option::<String>::deserialize(d)?;
    Ok(opt.map(|s| s.trim().to_string()))
}
