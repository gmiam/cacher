use http::request::Request;
use hyper::Body;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;

use crate::proxy::helpers::http_version_as_str;

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct ProxyRequest<'a> {
    pub method: String,
    version: &'a str,
    pub scheme: String,
    pub host: String,
    pub port: Option<String>,
    pub uri: String,
    pub headers: HashMap<String, String>,
}

impl<'a> From<&Request<Body>> for ProxyRequest<'a> {
    fn from(req: &Request<Body>) -> Self {
        let method = req.method().to_string();
        let version = http_version_as_str(req.version());
        let scheme = req.uri().scheme().unwrap().to_string();
        let host = req.uri().host().unwrap_or_default().to_string();
        let path = req.uri().to_string();
        let port = req.uri().port().map(|port| port.to_string());
        let uri = req.uri().path_and_query().map(|v| v.to_string()).unwrap_or(path);
        let headers: HashMap<String, String> = req.headers().iter().map(|(k, v)| (k.to_string(), String::from(v.to_str().unwrap_or("")))).collect();
        ProxyRequest { method, version, scheme, host, port, uri, headers }
    }
}

impl<'a> ProxyRequest<'a> {
    pub fn get_headers(&self) -> &HashMap<String,String> {
        &self.headers
    }
}

pub async fn get_proxy_uri(req: &Request<Body>, backend_host: &str) -> String {
    let path = req.uri().path();
    let path_query = req
        .uri()
        .path_and_query()
        .map(|v| v.as_str())
        .unwrap_or(path);
    format!("{}{}", backend_host, path_query)
}