use serde::Deserialize;
use std::time::Duration;

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    pub(crate) jwt: JwtConfig,
    pub(crate) session: SessionConfig,
}

#[derive(Debug, Deserialize)]
pub struct JwtConfig {
    pub(crate) secret_key: String,
    pub(crate) algorithm: String,
    pub(crate) expiry: String,
}

#[derive(Debug, Deserialize)]
pub struct SessionConfig {
    name: String,
    secure: bool,
    pub(crate) domain: String,
}

pub fn load_config() -> Result<AppConfig, Box<dyn std::error::Error>> {
    let content = std::fs::read_to_string("../auth-framework.toml")?;
    let mut config: AppConfig = toml::from_str(&content)?;

    // Resolve env-var placeholder: "${JWT_SECRET_KEY:development-secret}"
    if config.jwt.secret_key.starts_with("${") {
        let inner = config.jwt.secret_key
            .trim_start_matches("${")
            .trim_end_matches('}');

        let (env_var, default) = inner
            .split_once(':')
            .unwrap_or((inner, ""));

        config.jwt.secret_key = std::env::var(env_var)
            .unwrap_or_else(|_| default.to_string());
    }

    Ok(config)
}

pub fn parse_expiry(expiry: &str) -> Duration {
    // Handles simple formats like "1h", "30m", "3600s"
    let (num, unit) = expiry.split_at(expiry.len() - 1);
    let value: u64 = num.parse().unwrap_or(3600);
    match unit {
        "d" => Duration::from_secs(value * 86400),
        "h" => Duration::from_secs(value * 3600),
        "m" => Duration::from_secs(value * 60),
        "s" => Duration::from_secs(value),
        _ => Duration::from_secs(3600),
    }
}
