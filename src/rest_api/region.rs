use log::debug;
use std::collections::HashMap;
use warp;

use crate::spot_db;
use crate::spot_db::SharedDB;

use serde::{Deserialize, Serialize};

pub async fn get_region(
    name: String,
    shared_db: SharedDB,
) -> Result<impl warp::Reply, warp::Rejection> {
    debug!("--> get_db_region");
    let db = shared_db.read();
    if let Some(r) = db.get_region(&name) {
        let (band_activities, spotters, call_info) = r.get_band_activities();
        let region = Region {
            name,
            spotters,
            num_spotter_spots: r.spotter_spots.len(),
            band_activities,
            call_info,
        };
        Ok(warp::reply::json(&region))
    } else {
        Err(warp::reject::not_found())
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Frequency {
    pub callsigns: Vec<String>,
}

pub async fn get_frequency(
    freq_hz: u64,
    shared_db: SharedDB,
) -> Result<impl warp::Reply, warp::Rejection> {
    debug!("--> get_frequency");
    let db = shared_db.read();
    let freq_khz = (freq_hz as f64) / 1000.0;
    let callsigns = db.get_frequency_users(freq_khz);
    Ok(warp::reply::json(&Frequency { callsigns }))
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Regions {
    pub names: Vec<String>,
}

pub async fn get_regions(shared_db: SharedDB) -> Result<impl warp::Reply, warp::Rejection> {
    println!("handling regions request");
    let db = shared_db.read();
    let mut names: Vec<String> = db.get_regions().iter().map(|r| r.name.clone()).collect();
    names.sort_unstable();
    Ok(warp::reply::json(&Regions { names }))
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Region {
    pub name: String,
    pub spotters: Vec<String>,
    pub num_spotter_spots: usize,
    pub band_activities: Vec<spot_db::BandActivity>,
    pub call_info: HashMap<String, spot_db::CallInfo>,
}
