use crate::{BACKEND_HOST, HANDLE_VARY, REDIS_URL};



#[derive(Clone)]
pub struct CacherConfig {
    pub backend_host: String,
    pub handle_vary: bool,
    pub redis_url: String,
}

impl CacherConfig {
    pub fn new() -> Self {
        let backend_host = std::env::var("CACHER_BACKEND").unwrap_or(BACKEND_HOST.to_string());
        let env_handle_vary = std::env::var("CACHER_VARY").unwrap_or(HANDLE_VARY.to_string().to_ascii_lowercase());
        let handle_vary = match env_handle_vary.as_str() {
            "true" => true,
            "false" => false,
            _ => false,
        };
        let redis_url = std::env::var("CACHER_REDIS").unwrap_or(REDIS_URL.to_string());

        CacherConfig { backend_host, handle_vary, redis_url }
    }

    pub fn get_backend(&self) -> &str {
        self.backend_host.as_str()
    }

    pub fn get_redis(&self) -> &str {
        self.redis_url.as_str()
    }
}