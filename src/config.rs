use serde::Deserialize;
use std::{fs, path::Path};

#[derive(Debug, Deserialize)]
pub struct Config {
    pub host: String,
    pub port: i32,
}

pub fn load_config<P: AsRef<Path>>(path: P) -> Result<Config, String> {
    let text = fs::read_to_string(path).map_err(|e| e.to_string())?;
    let cfg: Config = serde_yaml::from_str(&text).map_err(|e| e.to_string())?;
    Ok(cfg)
}
