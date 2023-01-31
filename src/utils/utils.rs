use chrono::{prelude::DateTime, Utc};
use std::time::{Duration, UNIX_EPOCH};

// Convert a Win32 FILETIME to a Unix timestamp
pub fn filetime_to_unix_timestamp(filetime: u64) -> u64 {
    (filetime as u64 - 116444736000000000) / 10
}

// Get formatted timestamp string
pub fn get_timestamp_str(timestamp: u64) -> String {
    let d = UNIX_EPOCH + Duration::from_micros(timestamp);
    let datetime = DateTime::<Utc>::from(d);
    // If datetime is 1 year or more in the past, show the year
    if datetime < Utc::now() - chrono::Duration::days(365) {
        datetime.format("%b %e  %Y").to_string()
    } else {
        datetime.format("%b %e %H:%M").to_string()
    }
}
