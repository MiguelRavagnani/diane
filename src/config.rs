use serde::{Deserialize, Deserializer};
use std::path::PathBuf;

fn default_root() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".diane")
}

fn root_or_default<'de, D>(d: D) -> Result<PathBuf, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(d)?;
    let s = s.trim();
    Ok(if s.is_empty() {
        default_root()
    } else {
        PathBuf::from(s)
    })
}

#[derive(Deserialize)]
pub struct Config {
    #[serde(
        rename = "root",
        default = "default_root",
        deserialize_with = "root_or_default"
    )]
    pub diane_root: PathBuf,

    #[serde(default)]
    pub theme: String,
}
