use std::{env, time::Duration};

use crate::backend::BackendConfig;

#[derive(Clone, Debug)]
pub struct Config {
    pub api_host: String,
    pub api_port: u16,
    pub backend_url: String,
    pub api_key: Option<String>,
    pub max_text_chars: usize,
    pub max_concurrent: usize,
    pub request_timeout: Duration,
    pub backend: BackendConfig,
}

impl Config {
    pub fn load() -> Result<Self, String> {
        let api_host = env::var("API_HOST").unwrap_or_else(|_| "127.0.0.1".into());
        let api_port = parse("API_PORT", 7101)?;
        let backend_port = parse("BACKEND_PORT", 8101)?;
        let backend_url = env::var("BACKEND_URL")
            .unwrap_or_else(|_| format!("http://127.0.0.1:{backend_port}"))
            .trim_end_matches('/')
            .to_string();
        Ok(Self {
            api_host,
            api_port,
            backend_url,
            api_key: env::var("API_KEY").ok().filter(|v| !v.is_empty()),
            max_text_chars: parse("MAX_TEXT_CHARS", 5_000)?,
            max_concurrent: parse("MAX_CONCURRENT_SYNTHESIS", 1)?,
            request_timeout: Duration::from_secs(parse("REQUEST_TIMEOUT_SECONDS", 300)?),
            backend: BackendConfig::from_env(backend_port),
        })
    }
}

fn parse<T>(name: &str, default: T) -> Result<T, String>
where
    T: std::str::FromStr + std::fmt::Display,
{
    match env::var(name) {
        Ok(value) => value
            .parse()
            .map_err(|_| format!("invalid {name}: {value}")),
        Err(_) => Ok(default),
    }
}
