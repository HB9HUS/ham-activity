use super::region::{get_frequency, get_region, get_regions};
use super::stats::get_db_stats;
use crate::spot_db::SharedDB;
use log::info;
use warp;
use warp::Filter;

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

fn routes(
    db: SharedDB,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    let fs = warp::path("ui").and(warp::fs::dir("./static/ui/"));
    get_region_route(db.clone())
        .or(get_db_stats_route(db.clone()))
        .or(get_regions_route(db.clone()))
        .or(get_frequency_route(db.clone()))
        .or(fs)
}

fn with_db(
    db: SharedDB,
) -> impl Filter<Extract = (SharedDB,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || db.clone())
}

pub async fn serve(db: SharedDB) {
    let routes = routes(db);
    info!("Server started at http://0.0.0.0:8000");
    warp::serve(routes).run(([0, 0, 0, 0], 8000)).await;
}
