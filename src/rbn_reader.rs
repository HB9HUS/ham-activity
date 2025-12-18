use crate::config;
use crate::line_source::{LineSource, MockTelnet, RealTelnet};
use crate::spot_db::SharedDB;
use anyhow::{anyhow, bail, Result};
use chrono::LocalResult::Single;
use chrono::{DateTime, Datelike, Duration, TimeZone, Timelike, Utc};
use std::fs;
use std::io::BufRead;

struct SpotInfo {
    spotter: String,
    freq_khz: f64,
    spotted: String,
    mode: String,
    snr_db: u32,
    wpm: u32,
    msg: String,
    utc_time: String,
}

pub fn parse_hhmmz_to_utc(hhmmz: &str) -> Result<DateTime<Utc>> {
    if hhmmz.len() != 5 {
        bail!("String malformed, expect hhmmz, got {}", hhmmz);
    }
    let hr = hhmmz[0..2]
        .parse()
        .map_err(|e| anyhow!("could not parse hours in {hhmmz}: {e}"))?;
    let min = hhmmz[2..4]
        .parse()
        .map_err(|e| anyhow!("could not parse minutes in {hhmmz}: {e}"))?;
    let n = Utc::now();
    // did we just cross midnight?
    let n = if hr == 23 && n.hour() == 0 {
        n - Duration::days(1)
    } else {
        n
    };
    if let Single(t) = Utc.with_ymd_and_hms(n.year(), n.month(), n.day(), hr, min, 0) {
        Ok(t)
    } else {
        bail!("Could not convert time")
    }
}

fn parse_spot_split(line: &str) -> Option<SpotInfo> {
    // 1️⃣ Split the line into the “words” we care about.
    //    The iterator yields:
    //    ["DX","de","G4IRN-#:","3531.9","DL2AWA","CW","14","dB","23","WPM","CQ","2034Z"]
    let mut parts = line.split_ascii_whitespace();

    if parts.next()? != "DX" {
        return None;
    }
    if parts.next()? != "de" {
        return None;
    }

    let raw_origin = parts.next()?; // e.g. "G4IRN-#:"
    let spotter = raw_origin
        .trim_end_matches("-#:") // remove the suffix
        .trim_end_matches('-') // just in case the colon is missing
        .to_string();

    let freq_khz: f64 = parts.next()?.parse().ok()?;

    let spotted = parts.next()?.to_string();

    let mode = parts.next()?.to_string();

    let snr_db: u32 = parts.next()?.parse().ok()?;
    // skip the “dB” token
    if parts.next()? != "dB" {
        return None;
    }

    let wpm: u32 = parts.next()?.parse().ok()?;
    // skip the “WPM” token
    if parts.next()? != "WPM" {
        return None;
    }

    let msg = parts.next()?.to_string();

    let utc_time = parts.next()?.to_string();

    // Anything left is ignored

    Some(SpotInfo {
        spotter,
        freq_khz,
        spotted,
        mode,
        snr_db,
        wpm,
        msg,
        utc_time,
    })
}

pub async fn read_rbn(shared_db: SharedDB, cfg: config::RBNConfig) -> Result<()> {
    let mut rbn: Box<dyn LineSource> = if cfg.enable_test {
        let path = cfg.rbn_data_file;
        let rbn_data = fs::read_to_string(&path)
            .map_err(|e| anyhow!("could not read rbn_capture file {path}: {e}"))?;
        Box::new(MockTelnet::from_bytes_with_delay(
            rbn_data.as_bytes(),
            std::time::Duration::from_millis(500),
        ))
    } else {
        let rt = RealTelnet::connect(&cfg.host, cfg.port)?;
        Box::new(rt)
    };

    rbn.send_callsign(&cfg.callsign)?;

    let mut line = String::new();
    loop {
        line.clear();
        // read_until stops at '\n'; Telnet lines end with "\r\n"
        match rbn.read_line(&mut line) {
            Ok(0) => bail!("EOF"), // EOF
            Ok(_) => {
                if let Some(s) = parse_spot_split(line.as_str()) {
                    if let Ok(ts) = parse_hhmmz_to_utc(&s.utc_time) {
                        let mut db = shared_db.write();
                        db.add_spot(
                            &s.spotter, &s.spotted, s.freq_khz, &s.mode, s.snr_db, s.wpm, &s.msg,
                            ts,
                        );
                        println!("added spot {line}");
                    }
                } else {
                    eprintln!("could not parse line: {line}");
                }
            }
            Err(e) => {
                bail!("read error: {e}")
            }
        }
    }
}
