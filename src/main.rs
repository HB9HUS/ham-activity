use anyhow::Result;
use clap::Parser;
use log::{debug, error, info};
use std::path::PathBuf;
use std::process;
use std::time::Duration;
use tokio::spawn;

use rest_api::serve;

mod bands;
mod config;
mod line_source;
mod rbn_reader;
mod region_loader;
mod rest_api;
mod shared;
mod spot_db;

#[derive(Parser, Debug)]
#[command(
    name = "ham-activity",
    version,
    about = "Tool to fetch data from RBN and display in a simple UI and via Rest"
)]
struct Cli {
    /// Path to config file
    #[arg(short = 'c', long = "config", default_value = "config.yaml")]
    config: PathBuf,
    #[arg(short = 't', long = "test", default_value = "false")]
    test: bool,
}

async fn periodic_cleaner(shared_db: spot_db::SharedDB, db_cfg: config::DBConfig) {
    let cleanup_period = Duration::from_secs(db_cfg.cleanup_period_secs);
    let max_spot_age = Duration::from_secs(db_cfg.max_spot_age_secs);
    loop {
        tokio::time::sleep(cleanup_period).await;
        info!("running periodic cleaner");
        let mut db = shared_db.write();
        db.cleanup_old_spots(max_spot_age);
        info!("finished periodic cleaner");
    }
}
fn load_regions(shared_db: &spot_db::SharedDB, regions: &[region_loader::Dxcc]) {
    let mut db = shared_db.write();
    for region in regions {
        // any dxcc that has validEnd set is not relevant for us
        if !region.valid_end.is_empty() {
            continue;
        }
        let prefixes: Vec<String> = region
            .prefix
            .split(',')
            .map(std::string::ToString::to_string)
            .collect();
        db.add_region(
            region.name.to_lowercase().replace(char::is_whitespace, "_"),
            prefixes.clone(),
        );
        for cq in region.cq.clone() {
            let cq_name = format!("CQ_{cq}");
            match db.get_region(&cq_name) {
                Some(r) => {
                    let mut new_prefixes = prefixes.clone();
                    new_prefixes.append(&mut r.prefixes.clone());
                    db.add_region(cq_name, new_prefixes);
                }
                None => db.add_region(cq_name, prefixes.clone()),
            }
        }
        for continent in region.continent.clone() {
            match db.get_region(&continent) {
                Some(r) => {
                    let mut new_prefixes = prefixes.clone();
                    new_prefixes.append(&mut r.prefixes.clone());
                    db.add_region(continent, new_prefixes);
                }
                None => db.add_region(continent, prefixes.clone()),
            }
        }
    }
}

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let cli = Cli::parse();

    let mut cfg = match config::load_config(&cli.config) {
        Ok(cfg) => cfg,
        Err(e) => {
            error!("could not load config: {e}");
            process::exit(1)
        }
    };
    if cli.test {
        cfg.rbn.enable_test = true;
    }
    env_logger::init();
    debug!("{cfg:#?}");

    let shared_db = shared::Shared::new(spot_db::SpotDB::new());
    let regions = region_loader::load(cfg.region_file)?;

    load_regions(&shared_db.clone(), &regions);

    spawn(serve(shared_db.clone()));
    spawn(periodic_cleaner(shared_db.clone(), cfg.db));
    rbn_reader::read_rbn(shared_db.clone(), cfg.rbn).await?;

    Ok(())
}
