use volo_http::http::header::{HeaderName, HeaderValue, LOCATION, SET_COOKIE};
use volo_http::http::StatusCode;
use volo_http::response::Response;
use volo_http::server::IntoResponse;

pub fn redirect(location: &str) -> Response {
    let mut resp = Response::default();
    *resp.status_mut() = StatusCode::SEE_OTHER;
    if let Ok(value) = HeaderValue::from_str(location) {
        resp.headers_mut().insert(LOCATION, value);
    }
    resp
}

pub fn redirect_with_cookie(location: &str, cookie: &str) -> Response {
    let mut resp = redirect(location);
    if let Ok(value) = HeaderValue::from_str(cookie) {
        resp.headers_mut().insert(SET_COOKIE, value);
    }
    resp
}

pub fn with_header<R: IntoResponse>(resp: R, name: HeaderName, value: HeaderValue) -> Response {
    let mut resp = resp.into_response();
    resp.headers_mut().insert(name, value);
    resp
}
