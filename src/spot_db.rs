use crate::shared;
use chrono::{DateTime, Utc};
use core::ops::Sub;
use log::error;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

pub type SharedDB = shared::Shared<SpotDB>;

#[derive(Debug, PartialEq)]
pub struct Spot {
    pub spotter: String, // e.g. "G4IRN"
    pub spotted: String, // spotted callsign
    pub freq_khz: f64,   // frequency in kHz (or MHz – whatever the cluster reports)
    mode: String,        // CW, SSB, FT8 …
    snr_db: i32,         // signal‑to‑noise ratio, dB
    wpm: u32,            // words‑per‑minute
    msg: String,         // usually "CQ"
    pub timestamp: DateTime<Utc>,
}

pub struct Region {
    pub name: String,
    pub spotter_spots: Vec<Arc<Spot>>,
    pub prefixes: Vec<String>,
}

impl Region {
    pub fn new(name: String, prefixes: Vec<String>) -> Self {
        let spots = Vec::new();
        Self {
            name,
            spotter_spots: spots,
            prefixes,
        }
    }
    pub fn match_callsign(&self, callsign: &str) -> bool {
        self.prefixes.iter().any(|p| callsign.starts_with(p))
    }
    pub fn add_spot(&mut self, spot: Arc<Spot>) {
        if self.match_callsign(&spot.spotter) {
            self.spotter_spots.push(spot);
        }
    }
    pub fn remove_spots(&mut self, spots: &Vec<Arc<Spot>>) {
        spots
            .iter()
            .for_each(|remove_spot| self.spotter_spots.retain(|s| *s != *remove_spot));
    }
}

pub struct SpotDB {
    pub init_timestamp: DateTime<Utc>,
    spots: Vec<Arc<Spot>>,
    regions: HashMap<String, Region>,
}

impl SpotDB {
    pub fn new() -> Self {
        let spots = Vec::new();
        let regions = HashMap::new();
        Self {
            init_timestamp: Utc::now(),
            spots,
            regions,
        }
    }

    pub fn add_spot(
        &mut self,
        spotter: &str,
        spotted: &str,
        freq_khz: f64,
        mode: &str,
        snr_db: i32,
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
        // sanity check
        expired.iter().for_each(|e| {
            if Arc::strong_count(e) > 1 {
                error!("Bug in cleanup somewhere! Arc::strong_count > 1 after delete!")
            }
        })
    }

    pub fn spots_in_db(&self) -> usize {
        self.spots.len()
    }

    pub fn regions_in_db(&self) -> usize {
        self.regions.len()
    }

    pub fn add_region(&mut self, name: String, prefixes: Vec<String>) {
        let r = Region::new(name.clone(), prefixes);
        self.regions.insert(name, r);
    }

    pub fn get_region(&self, name: &str) -> Option<&Region> {
        self.regions.get(name)
    }

    pub fn get_regions(&self) -> Vec<&Region> {
        self.regions.iter().map(|(_, r)| r).collect()
    }

    pub fn get_frequency_users(&self, freq_khz: f64) -> Vec<String> {
        let mut callsigns: Vec<String> = self
            .spots
            .iter()
            .filter(|&s| (freq_khz + 0.2) >= s.freq_khz && (freq_khz - 0.2) <= s.freq_khz)
            .map(|s| s.spotted.clone())
            .collect();
        callsigns.sort_unstable();
        callsigns.dedup();
        callsigns
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
            assert_eq!(reg.spotter_spots.len(), 1)
        } else {
            panic!("did not get a region")
        }
    }
}
