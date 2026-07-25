//! All schedule math is pinned to Asia/Kolkata (IST, UTC+5:30, no DST)
//! regardless of the machine's system locale/timezone, since the plan is
//! built around Bengaluru calendar days. IST never observes daylight
//! saving, so a fixed offset is correct and avoids pulling in a full
//! timezone database dependency.

use chrono::{FixedOffset, NaiveDate, NaiveTime, Utc};

pub fn ist_offset() -> FixedOffset {
    FixedOffset::east_opt(5 * 3600 + 30 * 60).expect("valid fixed offset")
}

pub fn today_ist() -> NaiveDate {
    Utc::now().with_timezone(&ist_offset()).date_naive()
}

pub fn now_ist_time() -> NaiveTime {
    Utc::now().with_timezone(&ist_offset()).time()
}

pub fn parse_hhmm(value: &str) -> Option<NaiveTime> {
    NaiveTime::parse_from_str(value, "%H:%M").ok()
}
