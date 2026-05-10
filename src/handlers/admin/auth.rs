use serde::Deserialize;
use volo_http::http::header::{HeaderValue, LOCATION, SET_COOKIE};
use volo_http::http::StatusCode;
use volo_http::response::Response;
use volo_http::server::extract::Form;
use volo_http::server::IntoResponse;

use crate::app_state;
use crate::services::admin_guard;
use crate::services::auth_service;
use crate::templates::{AdminLoginTemplate, HtmlTemplate};
use crate::utils::cookie::{clear_session_cookie, parse_cookies, session_cookie, SESSION_COOKIE};
use crate::utils::error::AppError;
use crate::utils::extract::RequestHeaders;
use crate::utils::response::redirect;

#[derive(Debug, Deserialize)]
pub struct LoginForm {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct LogoutForm {
    pub csrf_token: String,
}

pub async fn login_form() -> Result<HtmlTemplate<AdminLoginTemplate>, AppError> {
    let state = app_state()?;
    Ok(HtmlTemplate(AdminLoginTemplate {
        site_name: state.settings.site_name.clone(),
        site_description: state.settings.site_description.clone(),
        error: None,
        username: String::new(),
    }))
}

pub async fn login_submit(Form(form): Form<LoginForm>) -> Response {
    let state = match app_state() {
        Ok(state) => state,
        Err(e) => return e.into_response(),
    };

    match auth_service::login(&state.pool, &form.username, &form.password).await {
        Ok(session) => {
            let cookie =
                session_cookie(&session.id, auth_service::SESSION_LIFETIME_SECS);
            let mut resp = redirect("/admin");
            if let Ok(value) = HeaderValue::from_str(&cookie) {
                resp.headers_mut().insert(SET_COOKIE, value);
            }
            resp
        }
        Err(AppError::Unauthorized) => {
            let template = AdminLoginTemplate {
                site_name: state.settings.site_name.clone(),
                site_description: state.settings.site_description.clone(),
                error: Some("Invalid username or password.".to_string()),
                username: form.username,
            };
            (StatusCode::UNAUTHORIZED, HtmlTemplate(template)).into_response()
        }
        Err(AppError::TooManyRequests { retry_after_secs }) => {
            let template = AdminLoginTemplate {
                site_name: state.settings.site_name.clone(),
                site_description: state.settings.site_description.clone(),
                error: Some(format!(
                    "Too many failed attempts. Try again in {retry_after_secs}s."
                )),
                username: form.username,
            };
            (StatusCode::TOO_MANY_REQUESTS, HtmlTemplate(template)).into_response()
        }
        Err(other) => other.into_response(),
    }
}

pub async fn logout(
    RequestHeaders(headers): RequestHeaders,
    Form(form): Form<LogoutForm>,
) -> Response {
    let state = match app_state() {
        Ok(state) => state,
        Err(e) => return e.into_response(),
    };

    let auth = match admin_guard::require_admin(&state.pool, &headers).await {
        Ok(auth) => auth,
        Err(error) => return error.into_response(),
    };
    if let Err(error) = admin_guard::verify_csrf(&auth, Some(&form.csrf_token)) {
        return error.into_response();
    }

    if let Some(session_id) = parse_cookies(&headers).get(SESSION_COOKIE) {
        if let Err(error) = auth_service::logout(&state.pool, session_id).await {
            return error.into_response();
        }
    }

    let mut resp = Response::default();
    *resp.status_mut() = StatusCode::SEE_OTHER;
    resp.headers_mut()
        .insert(LOCATION, HeaderValue::from_static("/admin/login"));
    if let Ok(cookie) = HeaderValue::from_str(&clear_session_cookie()) {
        resp.headers_mut().insert(SET_COOKIE, cookie);
    }
    resp
}
