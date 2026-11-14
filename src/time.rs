//! Tiny UTC clock helpers. The generator only needs ISO dates for the cache
//! stamp and for the GraphQL contribution window, so this avoids pulling a
//! date crate into the build.

use std::time::{SystemTime, UNIX_EPOCH};

pub fn now_unix() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

pub fn iso_utc(unix: i64) -> String {
    let (y, m, d) = date_parts(unix);
    let secs = unix.rem_euclid(86_400);
    format!(
        "{y:04}-{m:02}-{d:02}T{:02}:{:02}:{:02}Z",
        secs / 3600,
        (secs % 3600) / 60,
        secs % 60
    )
}

pub fn date_utc(unix: i64) -> String {
    let (y, m, d) = date_parts(unix);
    format!("{y:04}-{m:02}-{d:02}")
}

fn date_parts(unix: i64) -> (i64, u32, u32) {
    civil_from_days(unix.div_euclid(86_400))
}

/// Howard Hinnant's `civil_from_days`, valid for the whole range we care about.
fn civil_from_days(z: i64) -> (i64, u32, u32) {
    let z = z + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    (if m <= 2 { y + 1 } else { y }, m as u32, d as u32)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn epoch_is_1970() {
        assert_eq!(iso_utc(0), "1970-01-01T00:00:00Z");
    }

    #[test]
    fn known_dates_are_exact() {
        // 2026-11-13T00:00:00Z
        assert_eq!(date_utc(1_794_528_000), "2026-11-13");
        assert_eq!(iso_utc(1_794_528_000), "2026-11-13T00:00:00Z");
        // Leap day.
        assert_eq!(date_utc(1_709_164_800), "2024-02-29");
    }
}
