mod error;
mod proxy_response;
mod proxy_request;
mod cache;
mod proxy;
mod config;

use axum::{
    http::{uri::Uri, Request, Response},
    routing::get,
    Router, extract::State
};
use hyper::{client::HttpConnector, Body};
use r2d2::Pool;
use redis::Commands;
use std::{net::SocketAddr};
use std::time::{Instant};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use anyhow::Result;

use proxy_request::request::get_proxy_uri;
use cache::{CacheKey, CacheKeyNoVary, CacheKeyWithVary, cache_control::CacheControlRequest};
use config::CacherConfig;

use crate::{proxy::{response_from_origin_with_vary, response_from_origin_without_vary, response_from_cache, response_from_origin_without_cache}, cache::cache_control::CacheControlResponse};


type Client = hyper::client::Client<HttpConnector, Body>;

#[derive(Clone)]
pub struct ProxyState {
    http_client: hyper::client::Client<HttpConnector>,
    redis_pool: Pool<redis::Client>,
    config: CacherConfig,
}

const REDIS_URL: &str = "redis://127.0.0.1:6379/";
const BACKEND_HOST: &str = "http://stubr.rs:9191";
const HANDLE_VARY: bool = false;
const STATUS_HIT: &str = "HIT";
const STATUS_MISS: &str = "MISS";
const STATUS_DYNAMIC: &str = "DYNAMIC";

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG")
                .unwrap_or_else(|_| "example_http_proxy=trace,tower_http=debug".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();
    
    let config = CacherConfig::new();
    let redis_pool = get_redis_pool(&config).await.expect("Unable to create Redis connection pool");
    let http_client = Client::new();
    let state = ProxyState {http_client, redis_pool, config};

    let app = Router::with_state(state)
                        .route("/", get(proxy))
                        .route("/*path", get(proxy));

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    tracing::info!("reverse proxy listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .expect("Unable to launch proxy");
}

async fn proxy(State(state): State<ProxyState>, mut req: Request<Body>) -> Result<Response<Body>, error::ProxyError> {
    let start = Instant::now();
    let mut redis_conn = state.redis_pool.get()?;

    let duration = start.elapsed().as_micros();
    tracing::debug!("Time elapsed init {}µs", duration);

    // Replace host(format scheme://host:port) in incoming request URI with the host we want to proxify to
    let uri = get_proxy_uri(&req, state.config.get_backend()).await;
    *req.uri_mut() = Uri::try_from(uri)?;
    

    let cache_control = CacheControlRequest::try_from(&req).unwrap_or_default();
    tracing::info!("cache-control: {:?}", cache_control);
    // No Vary if disabled or Vary == "" or Vary == "*"
    // Enum cache status HIT - STALE - EXPIRED - MISS
    // if in cache
        // if < TTL -> HIT
        // if > TTL && < stale_TTL -> STALE
        // else -> EXPIRED
    // if not in cache -> MISS
    //if not cacheable -> DYNAMIC
    let no_cache = false;
    if no_cache {
        let response = response_from_origin_without_cache(req, state.http_client).await?;
        let duration = start.elapsed().as_micros();
        tracing::info!("Time elapsed HIT {}µs", duration);
        return Ok(response)
    };

    let vary_key = req.uri().path().to_ascii_lowercase();
    let cache_key = if state.config.handle_vary {
        let vary_content: Option<String> = redis_conn.get(&vary_key)?;
        let vary_content = vary_content.unwrap_or_else(|| "".to_string());
        CacheKeyWithVary::new_from_native(vary_content.as_str(), &req).get()
    } else {
        CacheKeyNoVary::from(&req).get()
    };

    let cached_response: Option<String> = redis_conn.get(&cache_key)?;

    if let Some(resp) = cached_response {
        let proxy_response = response_from_cache(resp).await?;
        let duration = start.elapsed().as_micros();
        tracing::info!("Time elapsed HIT {}µs", duration);
        Ok(proxy_response)
    } else {
        let proxy_response=  if state.config.handle_vary {
            response_from_origin_with_vary(req, state.http_client, redis_conn, vary_key).await?
        } else {
            response_from_origin_without_vary(req, state.http_client, redis_conn, cache_key).await?
        };
        let duration = start.elapsed().as_micros();
        tracing::info!("Time elapsed MISS {}µs", duration);
        let cache_control = CacheControlResponse::try_from(&proxy_response).unwrap_or_default();
        tracing::info!("cache-control: {:?}", cache_control);
        Ok(proxy_response)
    }
}   


async fn get_redis_pool(config: &CacherConfig) -> Result<Pool<redis::Client>> {
    let redis_client = redis::Client::open(config.get_redis())?;
    let pool = r2d2::Pool::builder().max_size(500).build(redis_client)?;
    Ok(pool)
}
