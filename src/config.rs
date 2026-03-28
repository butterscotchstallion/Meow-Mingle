use serde::Deserialize;
use std::fs;

#[derive(Debug, Deserialize, Clone)]
pub struct AppConfig {
    pub test_users: ConfigUsers,
    pub photo_upload_dir: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ConfigUsers {
    pub admin_username: String,
    pub admin_password: String,
    pub admin_id: String,
    pub unprivileged_username: String,
    pub unprivileged_password: String,
    pub unprivileged_id: String,
}

pub fn load_config() -> AppConfig {
    let filename = "app_config.toml";
    let contents =
        fs::read_to_string(filename).expect(&format!("Something went wrong reading {}", filename));
    toml::from_str(&contents).expect("Failed to parse app_config.toml")
}
