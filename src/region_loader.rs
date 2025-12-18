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
    country_code: String,

    /// CQ zones that belong to the entity
    pub cq: Vec<u8>,

    deleted: bool,

    #[serde(rename = "entityCode")]
    entity_code: u32,

    /// Emoji flag – stored as a plain string
    flag: String,

    /// ITU regions that belong to the entity
    itu: Vec<u8>,

    pub name: String,
    notes: String,

    #[serde(rename = "outgoingQslService")]
    outgoing_qsl_service: bool,

    /// Comma‑separated list of prefixes (e.g. "CF,CG,CH,…")
    pub prefix: String,

    #[serde(rename = "prefixRegex")]
    prefix_regex: String,

    #[serde(rename = "thirdPartyTraffic")]
    third_party_traffic: bool,

    #[serde(rename = "validEnd")]
    pub valid_end: String,

    #[serde(rename = "validStart")]
    valid_start: String,
}

pub fn load<P: AsRef<Path> + std::fmt::Display>(path: P) -> Result<Vec<Dxcc>> {
    let text = fs::read_to_string(&path).map_err(|e| anyhow!("failed to read {path}: {e}"))?;
    let dr: DxccRoot =
        serde_json::from_str(&text).map_err(|e| anyhow!("failed to read {path}: {e}"))?;
    Ok(dr.dxcc)
}
