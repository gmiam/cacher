use http::{Request, Response};
use hyper::Body;
use anyhow::{Result, Error};

#[derive(Clone, Debug, Default)]
pub struct CacheControlRequest {
    max_age: Option<String>,
    max_stale: Option<String>,
    min_fresh: Option<String>,
    no_cache: Option<String>,
    no_store: Option<String>,
    no_transform: Option<String>,
    only_if_cached: Option<String>,
    stale_if_error: Option<String>,
}

#[derive(Clone, Debug, Default)]
pub struct CacheControlResponse  {
    max_age: Option<String>,
    s_maxage: Option<String>,
    no_cache: Option<String>,
    no_store: Option<String>,
    no_transform: Option<String>,
    must_revalidate: Option<String>,
    proxy_revalidate: Option<String>,
    must_understand: Option<String>,
    private: Option<String>,
    public: Option<String>,
    immutable: Option<String>,
    stale_while_revalidate: Option<String>,
    stale_if_error: Option<String>,
}


impl TryFrom<&Request<Body>> for CacheControlRequest {
    type Error = Error;

    fn try_from(req: &Request<Body>) -> Result<Self, Self::Error> {
        let headers = req.headers();
        let cache_control = headers.get("cache-control").map(|it| it.to_str().unwrap_or(""));
        if let Some(content) = cache_control {
            let cache_control_content: Vec<&str> = content.split(',').map(|word| word.trim()).collect();
            let max_age = cache_control_content.iter().find(|&elem| elem.starts_with("max-age")).map(|content| content.to_lowercase());
            let max_stale = cache_control_content.iter().find(|&elem| elem.starts_with("max-stale")).map(|content| content.to_lowercase());
            let min_fresh = cache_control_content.iter().find(|&elem| elem.starts_with("min-fresh")).map(|content| content.to_lowercase());
            let no_cache = cache_control_content.iter().find(|&elem| elem.starts_with("no-cache")).map(|content| content.to_lowercase());
            let no_store = cache_control_content.iter().find(|&elem| elem.starts_with("no-store")).map(|content| content.to_lowercase());
            let no_transform = cache_control_content.iter().find(|&elem| elem.starts_with("no-transform")).map(|content| content.to_lowercase());
            let only_if_cached = cache_control_content.iter().find(|&elem| elem.starts_with("only-if-cached")).map(|content| content.to_lowercase());
            let stale_if_error = cache_control_content.iter().find(|&elem| elem.starts_with("stale-if-error")).map(|content| content.to_lowercase());
            Ok(CacheControlRequest { max_age, max_stale, min_fresh, no_cache, no_store, no_transform, only_if_cached, stale_if_error })
        } else {
            tracing::debug!("Error: No cache-control header");
            anyhow::bail!("No cache-control header");
        }
    }
}

impl TryFrom<&Response<Body>> for CacheControlResponse {
    type Error = Error;

    fn try_from(resp: &Response<Body>) -> Result<Self, Self::Error> {
        let headers = resp.headers();
        let cache_control = headers.get("cache-control").map(|it| it.to_str().unwrap_or(""));
        if let Some(content) = cache_control {
            let cache_control_content: Vec<&str> = content.split(',').map(|word| word.trim()).collect();
            let max_age = cache_control_content.iter().find(|&elem| elem.starts_with("max-age")).map(|content| content.to_lowercase());
            let s_maxage = cache_control_content.iter().find(|&elem| elem.starts_with("s-maxage")).map(|content| content.to_lowercase());
            let no_cache = cache_control_content.iter().find(|&elem| elem.starts_with("no-cache")).map(|content| content.to_lowercase());
            let no_store = cache_control_content.iter().find(|&elem| elem.starts_with("no-store")).map(|content| content.to_lowercase());
            let no_transform = cache_control_content.iter().find(|&elem| elem.starts_with("no-transform")).map(|content| content.to_lowercase());
            let must_revalidate = cache_control_content.iter().find(|&elem| elem.starts_with("must-revalidate")).map(|content| content.to_lowercase());
            let proxy_revalidate = cache_control_content.iter().find(|&elem| elem.starts_with("proxy-revalidate")).map(|content| content.to_lowercase());
            let must_understand = cache_control_content.iter().find(|&elem| elem.starts_with("must-understand")).map(|content| content.to_lowercase());
            let private = cache_control_content.iter().find(|&elem| elem.starts_with("private")).map(|content| content.to_lowercase());
            let public = cache_control_content.iter().find(|&elem| elem.starts_with("public")).map(|content| content.to_lowercase());
            let immutable = cache_control_content.iter().find(|&elem| elem.starts_with("immutable")).map(|content| content.to_lowercase());
            let stale_while_revalidate = cache_control_content.iter().find(|&elem| elem.starts_with("stale-while-revalidate")).map(|content| content.to_lowercase());
            let stale_if_error = cache_control_content.iter().find(|&elem| elem.starts_with("stale-if-error")).map(|content| content.to_lowercase());
            Ok(CacheControlResponse { max_age, s_maxage, no_cache, no_store, no_transform, must_revalidate, proxy_revalidate, must_understand, private, public, immutable, stale_while_revalidate, stale_if_error })
        } else {
            tracing::debug!("Error: No cache-control header");
            anyhow::bail!("No cache-control header");
        }
    }
}
