use std::collections::HashMap;

use volo_http::http::header::{HeaderMap, COOKIE};

pub const SESSION_COOKIE: &str = "vlog_session";

pub fn parse_cookies(headers: &HeaderMap) -> HashMap<String, String> {
    let mut out = HashMap::new();
    for value in headers.get_all(COOKIE).iter() {
        let Ok(text) = value.to_str() else { continue };
        for pair in text.split(';') {
            let pair = pair.trim();
            if pair.is_empty() {
                continue;
            }
            let Some(eq) = pair.find('=') else { continue };
            let (name, val) = pair.split_at(eq);
            let val = &val[1..];
            out.insert(name.trim().to_string(), val.trim().to_string());
        }
    }
    out
}

pub fn session_cookie(value: &str, max_age_secs: i64) -> String {
    if max_age_secs <= 0 {
        format!(
            "{}=; Path=/; HttpOnly; SameSite=Lax; Max-Age=0{}",
            SESSION_COOKIE,
            secure_suffix()
        )
    } else {
        format!(
            "{}={}; Path=/; HttpOnly; SameSite=Lax; Max-Age={}{}",
            SESSION_COOKIE,
            value,
            max_age_secs,
            secure_suffix()
        )
    }
}

pub fn clear_session_cookie() -> String {
    session_cookie("", 0)
}

fn secure_suffix() -> &'static str {
    match std::env::var("SESSION_COOKIE_SECURE") {
        Ok(value) if matches!(value.as_str(), "1" | "true" | "TRUE" | "yes" | "YES") => {
            "; Secure"
        }
        _ => "",
    }
}
