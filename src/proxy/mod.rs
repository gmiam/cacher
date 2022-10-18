use http::{Response, Request};
use http::header::HeaderName;
use anyhow::Result;
use hyper::{client::HttpConnector, Body};
use r2d2::PooledConnection;
use redis::{Commands, RedisError};

use crate::proxy_response::response::{ProxyResponse, FromResponse};
use crate::proxy_request::request::{ProxyRequest};
use crate::cache::{CacheKey, CacheKeyWithVary};
use crate::{error, STATUS_HIT, STATUS_MISS, STATUS_DYNAMIC};

pub(crate) mod helpers;


async fn add_header(mut response: Response<Body>, key: &str, value: Option<&str>) -> Result<Response<Body>> {
    if let Some(value) = value {
        let header_name = HeaderName::from_lowercase(key.as_bytes())?;
        let _ = response.headers_mut().append(header_name, value.parse()?);
        Ok(response)
    } else {
        anyhow::bail!("Failed to add header");
    }
}

pub async fn response_from_cache(resp: String) -> Result<Response<Body>, error::ProxyError> {
    let response: ProxyResponse = serde_json::from_str(resp.as_str())?;
    let mut proxy_response = Response::try_from(response)?;
    proxy_response = add_header(proxy_response, "cacher_status", Some(STATUS_HIT)).await?;
    Ok(proxy_response)
}

pub async fn response_from_origin_with_vary(req: Request<Body>, 
                http_client: hyper::client::Client<HttpConnector>, 
                mut redis_conn: PooledConnection<redis::Client>,
                vary_key: String) -> Result<Response<Body>, error::ProxyError> {

    let proxy_req = ProxyRequest::from(&req);

    let response = http_client.request(req).await?;
    let proxy_resp = ProxyResponse::from_resp(response).await;
    let vary_content = proxy_resp.headers.get("vary").unwrap_or(&String::default()).to_owned();
    //If key with vary not cached yet (do we want to revalidate?)
    let cache_key = CacheKeyWithVary::new_from_proxy(vary_content.as_str(), &proxy_req).get();

    let response_to_cache = serde_json::to_string(&proxy_resp)?;
    //Should be conditional. Do we cache? (vary == *, no cache-control, etc)
    let _: Result<String, RedisError> = redis_conn.set(&cache_key, response_to_cache);
    let _: Result<String, RedisError> = redis_conn.expire(&cache_key, 5); // TODO replace by real expiration value
    let _: Result<String, RedisError> = redis_conn.set(&vary_key, vary_content);

    let mut proxy_response = Response::try_from(proxy_resp)?;
    proxy_response = add_header(proxy_response, "cacher_status", Some(STATUS_MISS)).await?;

    Ok(proxy_response)
}


pub async fn response_from_origin_without_vary(req: Request<Body>, 
                http_client: hyper::client::Client<HttpConnector>, 
                mut redis_conn: PooledConnection<redis::Client>, 
                cache_key: String) -> Result<Response<Body>, error::ProxyError> {
                    
    let response = http_client.request(req).await?;
    let proxy_resp = ProxyResponse::from_resp(response).await;
    
    let response_to_cache = serde_json::to_string(&proxy_resp)?;
    //Should be conditional. Do we cache? (vary == *, no cache-control, etc)
    let _: Result<String, RedisError> = redis_conn.set(&cache_key, response_to_cache);
    let _: Result<String, RedisError> = redis_conn.expire(&cache_key, 5); // TODO replace by real expiration value

    let mut proxy_response = Response::try_from(proxy_resp)?;
    proxy_response = add_header(proxy_response, "cacher_status", Some(STATUS_MISS)).await?;

    Ok(proxy_response)
}

pub async fn response_from_origin_without_cache(req: Request<Body>, 
    http_client: hyper::client::Client<HttpConnector>) -> Result<Response<Body>, error::ProxyError> {
        
    let mut proxy_response = http_client.request(req).await?;
    proxy_response = add_header(proxy_response, "cacher_status", Some(STATUS_DYNAMIC)).await?;

    Ok(proxy_response)
}