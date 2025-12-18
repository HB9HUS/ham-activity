use clap::Parser;
use std::path::PathBuf;
use std::process;
use std::thread;
use std::time::Duration;
use tokio::spawn;

mod config;
mod line_source;
mod rbn_reader;
mod rest_api;
mod shared;
mod spot_db;

#[derive(Parser, Debug)]
#[command(
    name = "rbn-filter",
    version,
    about = "Filters and geneates stats from RBN"
)]

struct Cli {
    /// Path to config file
    #[arg(short = 'c', long = "config", default_value = "config.yaml")]
    config: PathBuf,
}

async fn periodic_cleaner(shared_db: spot_db::SharedDB, db_cfg: config::DBConfig) {
    let cleanup_period = Duration::from_secs(db_cfg.cleanup_period_secs);
    let max_spot_age = Duration::from_secs(db_cfg.max_spot_age_secs);
    loop {
        thread::sleep(cleanup_period);
        let mut db = shared_db.write();
        db.cleanup_old_spots(max_spot_age);
    }
}

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let cli = Cli::parse();

    let cfg = match config::load_config(cli.config) {
        Ok(cfg) => cfg,
        Err(e) => {
            eprintln!("could not load config: {}", e);
            process::exit(1)
        }
    };
    println!("{:#?}", cfg);

    let db = shared::Shared::new(spot_db::SpotDB::new());
    spawn(rest_api::serve(db.clone()));
    spawn(periodic_cleaner(db.clone(), cfg.db));
    rbn_reader::read_rbn(db.clone(), cfg.rbn).await?;

    let d = db.read();
    println!("spots: {}", d.spots_in_db());
    Ok(())
}
