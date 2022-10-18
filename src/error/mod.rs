
use anyhow::Error;
use axum::response::{IntoResponse, Response};
use http::StatusCode;


// Make our own error that wraps `anyhow::Error`.
pub struct ProxyError(Error);


// Tell axum how to convert `ProxyError` into a response.
impl IntoResponse for ProxyError {
    fn into_response(self) -> Response {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Something went wrong: {}", self.0),
        )
            .into_response()
    }
}

// This enables using `?` on functions that return `Result<_, anyhow::Error>` to turn them into
// `Result<_, ProxyError>`. That way you don't need to do that manually.
impl<E> From<E> for ProxyError
where
    E: Into<Error>,
{
    fn from(err: E) -> Self {
        Self(err.into())
    }
}