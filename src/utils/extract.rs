use std::convert::Infallible;

use volo_http::context::ServerContext;
use volo_http::http::header::HeaderMap;
use volo_http::http::request::Parts;
use volo_http::server::extract::FromContext;

#[derive(Debug, Clone, Default)]
pub struct RequestHeaders(pub HeaderMap);

impl FromContext for RequestHeaders {
    type Rejection = Infallible;

    async fn from_context(_: &mut ServerContext, parts: &mut Parts) -> Result<Self, Self::Rejection> {
        Ok(RequestHeaders(parts.headers.clone()))
    }
}
