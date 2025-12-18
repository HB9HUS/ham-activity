use crate::shared;
use chrono::{DateTime, Utc};
use core::ops::Sub;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

pub type SharedDB = shared::Shared<SpotDB>;

#[derive(Debug, PartialEq)]
struct Spot {
    spotter: String, // e.g. "G4IRN"
    spotted: String, // spotted callsign
    freq_khz: f64,   // frequency in kHz (or MHz – whatever the cluster reports)
    mode: String,    // CW, SSB, FT8 …
    snr_db: u32,     // signal‑to‑noise ratio, dB
    wpm: u32,        // words‑per‑minute
    msg: String,     // usually "CQ"
    timestamp: DateTime<Utc>,
}

struct Region {
    name: String,
    spots: Vec<Arc<Spot>>,
    prefixes: Vec<String>,
}

impl Region {
    pub fn new(name: String, prefixes: Vec<String>) -> Self {
        let spots = Vec::new();
        Self {
            name,
            spots,
            prefixes,
        }
    }
    pub fn match_callsign(&self, callsign: &str) -> bool {
        self.prefixes.iter().any(|p| callsign.starts_with(p))
    }
    pub fn add_spot(&mut self, spot: Arc<Spot>) {
        if self.match_callsign(&spot.spotted) {
            self.spots.push(spot);
        }
    }
    pub fn remove_spots(&mut self, spots: &Vec<Arc<Spot>>) {
        spots
            .iter()
            .for_each(|remove_spot| self.spots.retain(|s| *s != *remove_spot));
    }
}

pub struct SpotDB {
    spots: Vec<Arc<Spot>>,
    regions: HashMap<String, Region>,
}

impl SpotDB {
    pub fn new() -> Self {
        let spots = Vec::new();
        let regions = HashMap::new();
        Self { spots, regions }
    }

    pub fn add_spot(
        &mut self,
        spotter: &str,
        spotted: &str,
        freq_khz: f64,
        mode: &str,
        snr_db: u32,
        wpm: u32,
        msg: &str,
        timestamp: DateTime<Utc>,
    ) {
        let spot = Spot {
            spotter: spotter.to_string(),
            spotted: spotted.to_string(),
            freq_khz,
            mode: mode.to_string(),
            snr_db,
            wpm,
            msg: msg.to_string(),
            timestamp,
        };
        let s = Arc::new(spot);
        self.spots.push(s.clone());
        self.regions
            .iter_mut()
            .for_each(|(_, r)| r.add_spot(s.clone()));
    }

    pub fn cleanup_old_spots(&mut self, max_spot_age: Duration) {
        let cutoff = Utc::now().sub(max_spot_age);
        let (expired, active) = self
            .spots
            .iter()
            .cloned()
            .partition(|s| s.timestamp < cutoff);
        self.spots = active;
        self.regions
            .iter_mut()
            .for_each(|(_, r)| r.remove_spots(&expired));
    }

    pub fn spots_in_db(&self) -> usize {
        self.spots.len()
    }

    pub fn add_region(&mut self, name: String, prefixes: Vec<String>) {
        let r = Region::new(name.clone(), prefixes);
        self.regions.insert(name, r);
    }

    pub fn get_region(&self, name: &str) -> Option<&Region> {
        self.regions.get(name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::{fixture, rstest};

    #[fixture]
    fn empty_db() -> SpotDB {
        let db = SpotDB::new();
        db
    }

    #[rstest]
    fn init_db() {
        let db = SpotDB::new();
        assert_eq!(db.spots_in_db(), 0);
    }

    #[rstest]
    fn db_add_spot(mut empty_db: SpotDB) {
        empty_db.add_spot("HB9HUS", "HB9CL", 18080.0, "CW", 10, 25, "CQ", Utc::now());
        assert_eq!(empty_db.spots_in_db(), 1);
    }

    #[rstest]
    fn db_cleanup(mut empty_db: SpotDB) {
        let earlier = Utc::now() - Duration::from_secs(3600);
        empty_db.add_spot("HB9HUS", "HB9CL", 18080.0, "CW", 10, 25, "CQ", earlier);
        empty_db.cleanup_old_spots(Duration::from_secs(1000));
        assert_eq!(empty_db.spots_in_db(), 0);
    }

    #[rstest]
    fn db_add_get_region(mut empty_db: SpotDB) {
        let prefixes = vec!["HB".to_string(), "DL".to_string(), "F".to_string()];
        empty_db.add_region("europe".to_string(), prefixes);
        let r = empty_db.get_region("europe");
        if let Some(reg) = r {
            assert_eq!(reg.name, "europe")
        } else {
            panic!("did not get a region")
        }
    }
    #[rstest]
    fn db_add_spot_to_region(mut empty_db: SpotDB) {
        let prefixes = vec!["HB".to_string(), "DL".to_string(), "F".to_string()];
        empty_db.add_region("europe".to_string(), prefixes);
        empty_db.add_spot("HB9HUS", "DL1ABC", 18080.0, "CW", 10, 25, "CQ", Utc::now());
        let r = empty_db.get_region("europe");
        if let Some(reg) = r {
            assert_eq!(reg.spots.len(), 1)
        } else {
            panic!("did not get a region")
        }
    }
}
