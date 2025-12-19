use crate::bands::HF_BANDS;
use chrono::{DateTime, Utc};
use log::{debug, info};
use std::collections::HashMap;
use std::time::Duration;
use warp;
use warp::Filter;

use crate::spot_db;
use crate::spot_db::SharedDB;

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct DBStats {
    pub running_since: String,
    pub total_spots: usize,
    pub total_regions: usize,
}

async fn get_db_stats(shared_db: SharedDB) -> Result<impl warp::Reply, warp::Rejection> {
    debug!("--> get_db_stats");
    let db = shared_db.read();
    const FORMAT: &str = "%Y-%m-%d %H:%M:%S";
    let start_time = format!("{}", db.init_timestamp.format(FORMAT));
    let stats = DBStats {
        running_since: start_time,
        total_spots: db.spots_in_db(),
        total_regions: db.regions_in_db(),
    };
    Ok(warp::reply::json(&stats))
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Frequency {
    pub callsigns: Vec<String>,
}

async fn get_frequency(
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

async fn get_regions(shared_db: SharedDB) -> Result<impl warp::Reply, warp::Rejection> {
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
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct BandActivity {
    pub band: String,
    pub active_1min: Vec<String>,
    pub active_5min: Vec<String>,
    pub active_15min: Vec<String>,
}

async fn get_region(
    name: String,
    shared_db: SharedDB,
) -> Result<impl warp::Reply, warp::Rejection> {
    debug!("--> get_db_region");
    let db = shared_db.read();
    if let Some(r) = db.get_region(&name) {
        let (band_activities, spotters) = get_band_activities(r);
        let region = Region {
            name,
            spotters,
            num_spotter_spots: r.spotter_spots.len(),
            band_activities,
        };
        Ok(warp::reply::json(&region))
    } else {
        Err(warp::reject::not_found())
    }
}

fn get_band_activities(region: &spot_db::Region) -> (Vec<BandActivity>, Vec<String>) {
    debug!("--> get_band_activity");
    let mut band_activity = HashMap::new();
    let mut spotters = Vec::new();
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
                }
                if Utc::now() - Duration::from_secs(5 * 60) < spot.timestamp {
                    ba.active_5min.push(spot.spotted.clone());
                }
                if Utc::now() - Duration::from_secs(15 * 60) < spot.timestamp {
                    ba.active_15min.push(spot.spotted.clone());
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
    )
}

fn get_frequency_route(
    db: SharedDB,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("frequency" / u64)
        .and(warp::get())
        .and(with_db(db))
        .and_then(get_frequency)
}

fn get_regions_route(
    db: SharedDB,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("regions")
        .and(warp::get())
        .and(with_db(db))
        .and_then(get_regions)
}

fn get_region_route(
    db: SharedDB,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("region" / String)
        .and(warp::get())
        .and(with_db(db))
        .and_then(get_region)
}

fn get_db_stats_route(
    db: SharedDB,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("stats")
        .and(warp::get())
        .and(with_db(db))
        .and_then(get_db_stats)
}

pub fn routes(
    db: SharedDB,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    get_region_route(db.clone())
        .or(get_db_stats_route(db.clone()))
        .or(get_regions_route(db.clone()))
        .or(get_frequency_route(db.clone()))
}

fn with_db(
    db: SharedDB,
) -> impl Filter<Extract = (SharedDB,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || db.clone())
}

pub async fn serve(db: SharedDB) {
    let routes = routes(db);
    info!("Server started at http://localhost:8000");
    warp::serve(routes).run(([127, 0, 0, 1], 8000)).await;
}
