use log::debug;
use warp;

use crate::spot_db::SharedDB;

use serde::{Deserialize, Serialize};

pub async fn get_db_stats(shared_db: SharedDB) -> Result<impl warp::Reply, warp::Rejection> {
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
pub struct DBStats {
    pub running_since: String,
    pub total_spots: usize,
    pub total_regions: usize,
}
