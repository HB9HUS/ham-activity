use crate::bands::HF_BANDS;
use chrono::Utc;
use std::collections::HashMap;
use std::time::Duration;
use warp;
use warp::Filter;

use crate::spot_db;
use crate::spot_db::SharedDB;

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct DBStats {
    pub total_spots: usize,
    pub total_regions: usize,
}

async fn get_db_stats(shared_db: SharedDB) -> Result<impl warp::Reply, warp::Rejection> {
    let db = shared_db.read();
    let stats = DBStats {
        total_spots: db.spots_in_db(),
        total_regions: db.regions_in_db(),
    };
    Ok(warp::reply::json(&stats))
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Region {
    pub name: String,
    pub num_spotter_spots: usize,
    pub band_activities: Vec<BandActivity>,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct BandActivity {
    pub band: String,
    pub active_1min: usize,
    pub active_5min: usize,
    pub active_15min: usize,
}

async fn get_region(
    name: String,
    shared_db: SharedDB,
) -> Result<impl warp::Reply, warp::Rejection> {
    let db = shared_db.read();
    if let Some(r) = db.get_region(&name) {
        let band_activities = get_band_activities(r);
        let region = Region {
            name,
            num_spotter_spots: r.spotter_spots.len(),
            band_activities,
        };
        Ok(warp::reply::json(&region))
    } else {
        Err(warp::reject::not_found())
    }
}

fn get_band_activities(region: &spot_db::Region) -> Vec<BandActivity> {
    let mut band_activity = HashMap::new();
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
        for band in HF_BANDS {
            if band.lower_khz <= spot.freq_khz && spot.freq_khz <= band.upper_khz {
                let ba = band_activity
                    .get_mut(band.name)
                    .expect("initialized hashmap is missing entry!");
                if Utc::now() - Duration::from_secs(60) < spot.timestamp {
                    ba.active_1min += 1;
                }
                if Utc::now() - Duration::from_secs(5 * 60) < spot.timestamp {
                    ba.active_5min += 1;
                }
                if Utc::now() - Duration::from_secs(15 * 60) < spot.timestamp {
                    ba.active_15min += 1;
                }
            }
        }
    }
    HF_BANDS
        .iter()
        .map(|b| band_activity.get(b.name).unwrap().clone())
        .collect()
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
    get_region_route(db.clone()).or(get_db_stats_route(db.clone()))
}

fn with_db(
    db: SharedDB,
) -> impl Filter<Extract = (SharedDB,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || db.clone())
}

pub async fn serve(db: SharedDB) {
    let routes = routes(db);
    println!("Server started at http://localhost:8000");
    warp::serve(routes).run(([127, 0, 0, 1], 8000)).await;
}
