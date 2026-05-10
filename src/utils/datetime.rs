use chrono::{DateTime, NaiveDateTime, Utc};

const SQLITE_FORMATS: &[&str] = &[
    "%Y-%m-%d %H:%M:%S%.f",
    "%Y-%m-%d %H:%M:%S",
    "%Y-%m-%dT%H:%M:%S%.f",
    "%Y-%m-%dT%H:%M:%S",
];

pub fn parse(raw: &str) -> Option<DateTime<Utc>> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return None;
    }
    for fmt in SQLITE_FORMATS {
        if let Ok(naive) = NaiveDateTime::parse_from_str(trimmed, fmt) {
            return Some(DateTime::<Utc>::from_naive_utc_and_offset(naive, Utc));
        }
    }
    DateTime::parse_from_rfc3339(trimmed)
        .ok()
        .map(|dt| dt.with_timezone(&Utc))
}

pub fn rfc2822(raw: &str) -> String {
    parse(raw)
        .map(|dt| dt.to_rfc2822())
        .unwrap_or_else(|| raw.to_string())
}

pub fn rfc3339(raw: &str) -> String {
    parse(raw)
        .map(|dt| dt.to_rfc3339())
        .unwrap_or_else(|| raw.to_string())
}

pub fn iso_date(raw: &str) -> String {
    parse(raw)
        .map(|dt| dt.format("%Y-%m-%d").to_string())
        .unwrap_or_else(|| raw.to_string())
}

pub fn now_rfc2822() -> String {
    Utc::now().to_rfc2822()
}
