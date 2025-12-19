use crate::config;
use crate::line_source::{LineSource, MockTelnet, RealTelnet};
use crate::spot_db::SharedDB;
use anyhow::{anyhow, bail, Result};
use chrono::LocalResult::Single;
use chrono::{DateTime, Datelike, Duration, TimeZone, Timelike, Utc};
use log::{error, info, trace};
use std::fs;
use std::io::BufRead;
use std::io::ErrorKind;

struct SpotInfo {
    spotter: String,
    freq_khz: f64,
    spotted: String,
    mode: String,
    snr_db: i32,
    wpm: u32,
    msg: String,
    utc_time: DateTime<chrono::Utc>,
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

fn parse_spot_split(line: &str) -> Result<SpotInfo> {
    // 1️⃣ Split the line into the “words” we care about.
    //    The iterator yields:
    //    ["DX","de","G4IRN-#:","3531.9","DL2AWA","CW","14","dB","23","WPM","CQ","2034Z"]
    let mut parts = line.split_ascii_whitespace();

    // gets next part, checks if matches Some("want") if not None
    let mut get_part = |want| {
        let p = parts
            .next()
            .ok_or(anyhow! {"expected part, found nothing"})?;
        return match want {
            Some(wp) if p == wp => Ok(p),
            None => Ok(p),
            Some(wp) => bail!("expected {wp} found {p}"),
        };
    };

    get_part(Some("DX"))?;
    get_part(Some("de"))?;

    let raw_origin = get_part(None)?; // e.g. "G4IRN-#:"
    let spotter = raw_origin
        .trim_end_matches("-#:") // remove the suffix
        .trim_end_matches('-') // just in case the colon is missing
        .to_string();

    let freq_khz: f64 = get_part(None)?.parse()?;
    let spotted = get_part(None)?.to_string();
    let mode = get_part(None)?.to_string();
    let snr_db: i32 = get_part(None)?.parse()?;
    get_part(Some("dB"))?;
    let wpm: u32 = get_part(None)?.parse()?;
    get_part(Some("WPM"))?;
    let msg = get_part(None)?.to_string();
    if msg == "NCDXF" {
        // B probably means beacon?
        let _ = get_part(Some("B"));
    }
    let hhmmz = get_part(None)?.to_string();
    let utc_time = parse_hhmmz_to_utc(&hhmmz)?;

    // Anything left is ignored

    Ok(SpotInfo {
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

async fn connect_read(shared_db: SharedDB, cfg: &config::RBNConfig) -> Result<()> {
    let mut rbn: Box<dyn LineSource> = if cfg.enable_test {
        let path = cfg.rbn_data_file.clone();
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

    let mut line_buf = String::new();
    loop {
        line_buf.clear();
        // read_until stops at '\n'; Telnet lines end with "\r\n"
        match rbn.read_line(&mut line_buf) {
            Ok(0) => bail!("EOF"), // EOF
            Ok(_) => {
                let line = line_buf.trim();
                match parse_spot_split(line) {
                    Ok(s) => {
                        trace!("parsed: {line}");
                        let mut db = shared_db.write();
                        db.add_spot(
                            &s.spotter, &s.spotted, s.freq_khz, &s.mode, s.snr_db, s.wpm, &s.msg,
                            s.utc_time,
                        );
                    }
                    Err(e) => {
                        info!("could not parse line: {line}, {e}");
                    }
                }
            }
            Err(e) => {
                match e.kind() {
                    ErrorKind::WouldBlock | ErrorKind::TimedOut => {
                        // normal timeout, no data yet
                        // just continue the read loop
                        continue;
                    }
                    _ => bail!("read error: {e}"),
                }
            }
        }
    }
}

pub async fn read_rbn(shared_db: SharedDB, cfg: config::RBNConfig) -> Result<()> {
    loop {
        match connect_read(shared_db.clone(), &cfg).await {
            Ok(_) => error!("recieved Ok from connect read, should never happen!"),
            Err(e) if format!("{e}") == "EOF" => info!("got EOF, reconnecting"),
            Err(e) => info!("got {e}, reconnecting"),
        }
    }
}
