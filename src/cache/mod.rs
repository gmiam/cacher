pub mod cache_control;

use std::collections::BTreeMap;

use http::{Request};
use hyper::Body;

use crate::proxy_request::request::ProxyRequest;

pub trait CacheKey {
    fn get(self) -> String;
}

#[derive(Debug)]
pub struct CacheKeyNoVary {
    key: String,
}

#[derive(Debug)]
pub struct CacheKeyWithVary<'a> {
    uri: String,
    vary_headers: BTreeMap<String, &'a str>, // We need ordering
}
impl From<&Request<Body>> for CacheKeyNoVary {
    fn from(req: &Request<Body>) -> Self {
        let accept_lang = req.headers().get("Accept-Language").map(|header| header.to_str().unwrap_or("")   ).unwrap_or_else(|| "");
        let scheme = req.uri().scheme().map(|scheme| scheme.as_str()).unwrap_or("");
        let host = req.uri().host().unwrap_or("");
        let key = if let Some(port) = req.uri().port() {
            format!("{}_{}://{}:{}{}{}", req.method(), scheme, host, port, req.uri(), accept_lang)
        } else {
            format!("{}_{}://{}{}{}", req.method(), scheme, host, req.uri(), accept_lang)
        };
        CacheKeyNoVary { key }
    }
}

impl CacheKey for CacheKeyNoVary {
    fn get(self) -> String {
        self.key
    }
}

impl<'a> CacheKey for CacheKeyWithVary<'a> {
    fn get(self) -> String {
        let mut key = self.uri;
        self.vary_headers.iter().for_each(|(_, v)| key.insert_str(key.len(), v));
        key
    }  
}

impl<'a> CacheKeyWithVary<'a> {
    pub fn new_from_proxy(vary_content: &str, proxy_req: &'a ProxyRequest) -> Self {
        let uri = if let Some(port) = proxy_req.port.clone() {
            format!("{}_{}://{}:{}{}", proxy_req.method, proxy_req.scheme, proxy_req.host, port, proxy_req.uri)
        } else {
            format!("{}_{}://{}{}", proxy_req.method, proxy_req.scheme, proxy_req.host, proxy_req.uri)
        };
        let vary: Vec<&str> = vary_content.split(',').map(|word| word.trim()).collect();

        let mut vary_headers = BTreeMap::new();
        vary.iter().for_each(|&header| {
            let header_map = proxy_req.get_headers();
            let value = header_map.get(header).map(|it| it.as_str()).unwrap_or_else(|| "");
            vary_headers.insert(header.to_string(), value);
        });
        CacheKeyWithVary { uri, vary_headers }
    }

    pub fn new_from_native(vary_content: &str, req: &'a Request<Body>) -> Self {
        let scheme = req.uri().scheme().map(|scheme| scheme.as_str()).unwrap_or_else(|| "");
        let host = req.uri().host().unwrap_or("");
        let naked_path = req.uri().to_string();
        let path = req.uri().path_and_query().map(|v| v.to_string()).unwrap_or(naked_path);
        
        let uri = if let Some(port) = req.uri().port() {
            format!("{}_{}://{}:{}{}", req.method(), scheme, host, port, path)
        } else {
            format!("{}_{}://{}{}", req.method(), scheme, host, path)
        };
        let vary: Vec<&str> = vary_content.split(',').map(|word| word.trim()).collect();

        let mut vary_headers = BTreeMap::new();
        vary.iter().for_each(|&header| {
            let value = req.headers().get(header).map(|it| it.to_str().unwrap_or("")).unwrap_or_else(|| "");
            vary_headers.insert(header.to_string(), value);
        });
        CacheKeyWithVary { uri, vary_headers }
    }
}