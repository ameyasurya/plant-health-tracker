//! Local-calendar helpers.
//!
//! Earlier builds pinned all schedule math to Asia/Kolkata, because the
//! app was written for one Bengaluru balcony. Now that anyone can install
//! it, the day boundary follows the user's configured location instead --
//! someone in London should not have "today" roll over at 18:30 their
//! time.
//!
//! Falls back to the machine's own timezone when no location has been set
//! up yet, which is the right default for a fresh install.

use chrono::{Local, NaiveDate, NaiveTime, Utc};
use chrono_tz::Tz;

use crate::models::Location;

/// Resolves an IANA name to a timezone, falling back to the system zone
/// when the name is missing or unrecognised.
fn zone_of(location: Option<&Location>) -> Option<Tz> {
    location.and_then(|l| l.timezone.parse::<Tz>().ok())
}

pub fn today_local(location: Option<&Location>) -> NaiveDate {
    match zone_of(location) {
        Some(tz) => Utc::now().with_timezone(&tz).date_naive(),
        None => Local::now().date_naive(),
    }
}

pub fn now_local_time(location: Option<&Location>) -> NaiveTime {
    match zone_of(location) {
        Some(tz) => Utc::now().with_timezone(&tz).time(),
        None => Local::now().time(),
    }
}

pub fn parse_hhmm(value: &str) -> Option<NaiveTime> {
    NaiveTime::parse_from_str(value.trim(), "%H:%M").ok()
}

/// Southern-hemisphere seasons run six months out of phase with northern
/// ones, so the season lookup is fed a shifted month below the equator.
/// Coarse, but far more honest than assuming everyone shares a hemisphere.
pub fn season_month_for(latitude: Option<f64>, month: u32) -> u32 {
    match latitude {
        Some(lat) if lat < 0.0 => ((month + 5) % 12) + 1,
        _ => month,
    }
}
