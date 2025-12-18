use warp;
use warp::http::StatusCode;
use warp::Filter;

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
}

async fn get_region(
    name: String,
    shared_db: SharedDB,
) -> Result<impl warp::Reply, warp::Rejection> {
    let db = shared_db.read();
    if let Some(r) = db.get_region(&name) {
        let region = Region {
            name,
            num_spotter_spots: r.spotter_spots.len(),
        };
        Ok(warp::reply::json(&region))
    } else {
        Err(warp::reject::not_found())
    }
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
