use serde::Deserialize;
use std::{fs, path::Path};

#[derive(Debug, Deserialize)]
pub struct Config {
    pub rbn: RBNConfig,
    pub db: DBConfig,
}

#[derive(Debug, Deserialize)]
pub struct RBNConfig {
    pub callsign: String,
    pub host: String,
    pub port: u16,
    #[serde(default)]
    pub enable_test: bool,
    #[serde(default)]
    pub rbn_data_file: String,
}

#[derive(Debug, Deserialize)]
pub struct DBConfig {
    pub cleanup_period_secs: u64,
    pub max_spot_age_secs: u64,
}

pub fn load_config<P: AsRef<Path>>(path: P) -> Result<Config, String> {
    let text = fs::read_to_string(path).map_err(|e| e.to_string())?;
    let cfg: Config = serde_yaml::from_str(&text).map_err(|e| e.to_string())?;
    Ok(cfg)
}
