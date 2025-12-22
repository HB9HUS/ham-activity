use crate::bands::HF_BANDS;
use chrono::Utc;
use log::debug;
use std::collections::HashMap;
use std::time::Duration;
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
        let (band_activities, spotters, call_info) = get_band_activities(r);
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
    pub band_activities: Vec<BandActivity>,
    pub call_info: HashMap<String, CallInfo>,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct CallInfo {
    pub frequencies: Vec<f64>,
    pub wpm: Vec<u32>,
    pub db: Vec<i32>,
}

fn upser_call_info(spot: &spot_db::Spot, call_infos: &mut HashMap<String, CallInfo>) {
    match call_infos.entry(spot.spotted.clone()) {
        std::collections::hash_map::Entry::Occupied(mut occ) => {
            let orig = occ.get_mut();
            orig.frequencies.push(spot.freq_khz);
            orig.frequencies.sort_by(f64::total_cmp);
            orig.frequencies.dedup();
            orig.wpm.push(spot.wpm);
            orig.wpm.sort_unstable();
            orig.wpm.dedup();
            orig.db.push(spot.snr_db);
            orig.db.sort_unstable();
            orig.db.dedup();
        }
        std::collections::hash_map::Entry::Vacant(vac) => {
            let info = CallInfo {
                frequencies: vec![spot.freq_khz],
                wpm: vec![spot.wpm],
                db: vec![spot.snr_db],
            };
            vac.insert(info);
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct BandActivity {
    pub band: String,
    pub active_1min: Vec<String>,
    pub active_5min: Vec<String>,
    pub active_15min: Vec<String>,
}

fn get_band_activities(
    region: &spot_db::Region,
) -> (Vec<BandActivity>, Vec<String>, HashMap<String, CallInfo>) {
    debug!("--> get_band_activity");
    let mut band_activity = HashMap::new();
    let mut spotters = Vec::new();
    let mut call_info = HashMap::new();
    for band in HF_BANDS {
        band_activity.insert(
            band.name.to_string(),
            BandActivity {
                band: band.name.to_string(),
                ..Default::default()
            },
        );
    }
    for spot in region.spotter_spots.clone() {
        spotters.push(spot.spotter.clone());
        for band in HF_BANDS {
            if band.lower_khz <= spot.freq_khz && spot.freq_khz <= band.upper_khz {
                let ba = band_activity
                    .get_mut(band.name)
                    .expect("initialized hashmap is missing entry!");
                if Utc::now() - Duration::from_secs(60) < spot.timestamp {
                    ba.active_1min.push(spot.spotted.clone());
                    upser_call_info(&spot, &mut call_info);
                    continue; // only list the newest spot, ignore 5 and 15min
                }
                if Utc::now() - Duration::from_secs(5 * 60) < spot.timestamp {
                    ba.active_5min.push(spot.spotted.clone());
                    upser_call_info(&spot, &mut call_info);
                    continue; // ignore 15min
                }
                if Utc::now() - Duration::from_secs(15 * 60) < spot.timestamp {
                    ba.active_15min.push(spot.spotted.clone());
                    upser_call_info(&spot, &mut call_info);
                }
            }
        }
    }
    spotters.sort_unstable();
    spotters.dedup();
    // convert to vector in order of HF_BANDS
    // remove all duplicate callsigns
    (
        HF_BANDS
            .iter()
            .map(|b| {
                let mut ba = band_activity.get(b.name).unwrap().clone();
                ba.active_1min.sort_unstable();
                ba.active_1min.dedup();
                ba.active_5min.sort_unstable();
                ba.active_5min.dedup();
                ba.active_15min.sort_unstable();
                ba.active_15min.dedup();
                ba
            })
            .collect(),
        spotters,
        call_info,
    )
}
