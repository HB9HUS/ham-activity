use anyhow::Result;
use chrono::{DateTime, Utc};
use core::ops::Sub;
use dashmap::DashMap;
use once_cell::sync::Lazy;
use std::sync::Arc;
use std::time::Duration;

static CALLSIGNS: Lazy<DashMap<String, Arc<str>>> = Lazy::new(DashMap::new);

fn intern(cs: &str) -> Arc<str> {
    // `entry` gives us the “insert‑if‑absent” semantics without a lock.
    let entry = CALLSIGNS.entry(cs.to_owned());
    match entry {
        dashmap::mapref::entry::Entry::Occupied(o) => o.get().clone(),
        dashmap::mapref::entry::Entry::Vacant(v) => {
            let arc: Arc<str> = Arc::from(cs);
            v.insert(arc.clone());
            arc
        }
    }
}

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
        self.spots.push(spot);
    }
    pub fn remove_spots(&mut self, spots: &Vec<Arc<Spot>>) {
        spots
            .iter()
            .for_each(|remove_spot| self.spots.retain(|s| *s != *remove_spot));
    }
}

pub struct SpotDB {
    pub spots: Vec<Arc<Spot>>,
    pub regions: Vec<Region>,
}

impl SpotDB {
    pub fn new() -> Self {
        let spots = Vec::new();
        let regions = Vec::new();
        Self { spots, regions }
    }

    pub fn add_spot(
        &mut self,
        spotter: String,
        spotted: String,
        freq_khz: f64,
        mode: String,
        snr_db: u32,
        wpm: u32,
        msg: String,
        timestamp: DateTime<Utc>,
    ) {
        let spot = Spot {
            spotter,
            spotted,
            freq_khz,
            mode,
            snr_db,
            wpm,
            msg,
            timestamp,
        };
        let s = Arc::new(spot);
        self.spots.push(s);
    }

    pub fn cleanup_old_spots(&mut self, timeout: Duration) -> Result<()> {
        let cutoff = Utc::now().sub(timeout);
        let (expired, active) = self
            .spots
            .iter()
            .cloned()
            .partition(|s| s.timestamp < cutoff);
        self.spots = active;
        self.regions
            .iter_mut()
            .for_each(|r| r.remove_spots(&expired));
        Ok(())
    }
    pub fn spots_in_db(&self) -> usize {
        self.spots.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn init_db() {
        let db = SpotDB::new();
        assert_eq!(db.spots_in_db(), 0);
    }
}
