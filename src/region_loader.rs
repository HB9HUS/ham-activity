use anyhow::{anyhow, Result};
use serde::Deserialize;
use std::{fs, path::Path};

#[derive(Debug, Deserialize)]
struct DxccRoot {
    dxcc: Vec<Dxcc>,
}

#[derive(Debug, Deserialize)]
pub struct Dxcc {
    /// Example: ["NA"]
    pub continent: Vec<String>,

    /// ISO‑3166‑1 alpha‑2 country code (e.g. "CA")
    #[serde(rename = "countryCode")]
    _country_code: String,

    /// CQ zones that belong to the entity
    pub cq: Vec<u8>,

    #[serde(rename = "deleted")]
    _deleted: bool,

    #[serde(rename = "entityCode")]
    _entity_code: u32,

    /// Emoji flag – stored as a plain string
    #[serde(rename = "flag")]
    _flag: String,

    /// ITU regions that belong to the entity
    #[serde(rename = "itu")]
    _itu: Vec<u8>,

    pub name: String,

    #[serde(rename = "notes")]
    _notes: String,

    #[serde(rename = "outgoingQslService")]
    _outgoing_qsl_service: bool,

    /// Comma‑separated list of prefixes (e.g. "CF,CG,CH,…")
    pub prefix: String,

    #[serde(rename = "prefixRegex")]
    _prefix_regex: String,

    #[serde(rename = "thirdPartyTraffic")]
    _third_party_traffic: bool,

    #[serde(rename = "validEnd")]
    pub valid_end: String,

    #[serde(rename = "validStart")]
    _valid_start: String,
}

pub fn load<P: AsRef<Path> + std::fmt::Display>(path: P) -> Result<Vec<Dxcc>> {
    let text = fs::read_to_string(&path).map_err(|e| anyhow!("failed to read {path}: {e}"))?;
    let dr: DxccRoot =
        serde_json::from_str(&text).map_err(|e| anyhow!("failed to read {path}: {e}"))?;
    Ok(dr.dxcc)
}
