use crate::bands::HF_BANDS;
use crate::shared;
use chrono::{DateTime, Utc};
use core::ops::Sub;
use log::{debug, error};
use serde::{Deserialize, Serialize};
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
    pub snr_db: i32,     // signal‑to‑noise ratio, dB
    pub wpm: u32,        // words‑per‑minute
    msg: String,         // usually "CQ"
    pub timestamp: DateTime<Utc>,
}

pub struct Region {
    pub name: String,
    pub spotter_spots: Vec<Arc<Spot>>,
    pub prefixes: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct BandActivity {
    pub band: String,
    pub active_1min: Vec<String>,
    pub active_5min: Vec<String>,
    pub active_15min: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct CallInfo {
    pub frequencies: Vec<f64>,
    pub wpm: Vec<u32>,
    pub db: Vec<i32>,
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
    pub fn remove_spots(&mut self, spots: &[Arc<Spot>]) {
        for remove_spot in spots {
            self.spotter_spots.retain(|s| *s != *remove_spot);
        }
    }
    pub fn get_band_activities(
        &self,
    ) -> (Vec<BandActivity>, Vec<String>, HashMap<String, CallInfo>) {
        debug!("--> get_band_activity");
        let mut band_activity = HashMap::new();
        let mut spotters = Vec::new();
        let mut call_info = HashMap::new();
        for band in HF_BANDS {
            band_activity.insert(
                band.name.to_string(),
                BandActivity {
                    band: band.name.to_string(),
                    ..Default::default()
                },
            );
        }
        for spot in self.spotter_spots.clone() {
            spotters.push(spot.spotter.clone());
            for band in HF_BANDS {
                if band.lower_khz <= spot.freq_khz && spot.freq_khz <= band.upper_khz {
                    let ba = band_activity
                        .get_mut(band.name)
                        .expect("initialized hashmap is missing entry!");
                    if Utc::now() - Duration::from_secs(60) < spot.timestamp {
                        ba.active_1min.push(spot.spotted.clone());
                        upser_call_info(&spot, &mut call_info);
                        continue; // only list the newest spot, ignore 5 and 15min
                    }
                    if Utc::now() - Duration::from_secs(5 * 60) < spot.timestamp {
                        ba.active_5min.push(spot.spotted.clone());
                        upser_call_info(&spot, &mut call_info);
                        continue; // ignore 15min
                    }
                    if Utc::now() - Duration::from_secs(15 * 60) < spot.timestamp {
                        ba.active_15min.push(spot.spotted.clone());
                        upser_call_info(&spot, &mut call_info);
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
            call_info,
        )
    }
}

fn upser_call_info(spot: &Spot, call_infos: &mut HashMap<String, CallInfo>) {
    match call_infos.entry(spot.spotted.clone()) {
        std::collections::hash_map::Entry::Occupied(mut occ) => {
            let orig = occ.get_mut();
            orig.frequencies.push(spot.freq_khz);
            orig.frequencies.sort_by(f64::total_cmp);
            orig.frequencies.dedup();
            orig.wpm.push(spot.wpm);
            orig.wpm.sort_unstable();
            orig.wpm.dedup();
            orig.db.push(spot.snr_db);
            orig.db.sort_unstable();
            orig.db.dedup();
        }
        std::collections::hash_map::Entry::Vacant(vac) => {
            let info = CallInfo {
                frequencies: vec![spot.freq_khz],
                wpm: vec![spot.wpm],
                db: vec![spot.snr_db],
            };
            vac.insert(info);
        }
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

    // refactor another time
    #[allow(clippy::too_many_arguments)]
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
        for e in expired {
            if Arc::strong_count(&e) > 1 {
                error!("Bug in cleanup somewhere! Arc::strong_count > 1 after delete!");
            }
        }
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
        self.regions.values().collect()
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
        SpotDB::new()
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
