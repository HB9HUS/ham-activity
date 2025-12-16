use clap::Parser;
use std::path::PathBuf;
use std::process;
use std::sync::Arc;
use std::sync::RwLock;
use tokio::spawn;

mod config;
mod line_source;
mod rbn_reader;
mod rest_api;
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

    let db = Arc::new(RwLock::new(spot_db::SpotDB::new()));
    spawn(rest_api::serve(db.clone()));
    rbn_reader::read_rbn(db.clone()).await;

    if let Ok(d) = db.read() {
        println!("spots: {}", d.spots_in_db());
    }
    Ok(())
}
