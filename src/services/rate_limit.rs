use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use once_cell::sync::Lazy;

pub const FAILURE_LIMIT: u32 = 5;
pub const FAILURE_WINDOW: Duration = Duration::from_secs(60);
pub const LOCKOUT: Duration = Duration::from_secs(60);

#[derive(Debug, Default)]
struct AttemptState {
    count: u32,
    window_start: Option<Instant>,
    lockout_until: Option<Instant>,
}

static ATTEMPTS: Lazy<Mutex<HashMap<String, AttemptState>>> = Lazy::new(Default::default);

pub fn check(key: &str) -> Result<(), Duration> {
    let mut guard = ATTEMPTS.lock().expect("rate-limit mutex poisoned");
    let entry = guard.entry(key.to_string()).or_default();
    let now = Instant::now();
    if let Some(until) = entry.lockout_until {
        if until > now {
            return Err(until - now);
        }
        entry.lockout_until = None;
        entry.count = 0;
        entry.window_start = None;
    }
    Ok(())
}

pub fn record_failure(key: &str) -> Option<Duration> {
    let mut guard = ATTEMPTS.lock().expect("rate-limit mutex poisoned");
    let entry = guard.entry(key.to_string()).or_default();
    let now = Instant::now();

    if let Some(until) = entry.lockout_until {
        if until > now {
            return Some(until - now);
        }
        entry.lockout_until = None;
        entry.count = 0;
        entry.window_start = None;
    }

    let in_window = entry
        .window_start
        .is_some_and(|start| now.duration_since(start) < FAILURE_WINDOW);
    if in_window {
        entry.count += 1;
    } else {
        entry.count = 1;
        entry.window_start = Some(now);
    }

    if entry.count >= FAILURE_LIMIT {
        let until = now + LOCKOUT;
        entry.lockout_until = Some(until);
        Some(LOCKOUT)
    } else {
        None
    }
}

pub fn record_success(key: &str) {
    let mut guard = ATTEMPTS.lock().expect("rate-limit mutex poisoned");
    guard.remove(key);
}

#[allow(dead_code)]
pub fn reset_for_test(key: &str) {
    let mut guard = ATTEMPTS.lock().expect("rate-limit mutex poisoned");
    guard.remove(key);
}
