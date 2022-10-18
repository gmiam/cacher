use std::collections::HashMap;
use async_trait::async_trait;
use axum::response::Response;
use http::{response::{Parts, Builder}, StatusCode, HeaderValue, header::HeaderName, HeaderMap};
use hyper::Body;
use anyhow::{Result, Error};
use serde::{Deserialize, Serialize};
use std::fmt;
use crate::proxy::helpers::{get_http_version, http_version_as_str};


#[async_trait]
pub trait FromResponse<T>: Sized {
    async fn from_resp(resp: T) -> Self;
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct ProxyResponse<'a> {
    status: u16,
    version: &'a str,
    pub headers: HashMap<String, String>,
    body: String,
}

#[async_trait]
impl<'a> FromResponse<Response<Body>> for ProxyResponse<'a> {
    async fn from_resp(resp: Response<Body>) -> Self {
        let status = resp.status().as_u16();
        let (parts, body): (Parts, Body) = resp.into_parts();
        let version = http_version_as_str(parts.version);
        let headers: HashMap<String, String> = parts.headers.iter().map(|(k, v)| (k.to_string(), String::from(v.to_str().unwrap_or("")))).collect();
        let body = get_response_body_as_string(body).await.unwrap_or_else(|_| "".to_string());
        ProxyResponse { status, version, headers, body }
    }
}

impl<'a> TryFrom<ProxyResponse<'a>> for Response<Body> {
    type Error = Error;

    fn try_from (proxy_resp: ProxyResponse) -> Result<Self, Self::Error> {
        let status = StatusCode::from_u16(proxy_resp.status)?;
        let version = get_http_version(proxy_resp.version);
        let body = Body::from(proxy_resp.body);
        let mut builder = Builder::new().status(status).version(version);
        {
            let mut default_headers = HeaderMap::new();
            let headers = builder.headers_mut().unwrap_or(&mut default_headers);
            for (key, value) in proxy_resp.headers.into_iter() {
                headers.insert(HeaderName::try_from(&key)?, HeaderValue::from_bytes(value.as_bytes())?);
            }
        }
        let response = builder.body(body)?;
        Ok(response)
    }
}

impl<'a> fmt::Display for ProxyResponse<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", serde_json::to_string(&self).unwrap_or_default())
    }
}



pub async fn get_response_body_as_string<'a>(body: Body) -> Result<String> {
    let body_bytes = hyper::body::to_bytes(body).await?.to_vec();
    let body = String::from_utf8(body_bytes)?;
    Ok(body)
}
