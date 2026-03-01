use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tauri::{AppHandle, Manager};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserConfig {
    pub username: String,
    pub group: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppConfig {
    pub user: UserConfig,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            user: UserConfig {
                username: whoami::username(),
                group: "自己".to_string(),
            },
        }
    }
}

pub fn get_config_path(app: &AppHandle) -> PathBuf {
    let mut path = app.path().app_data_dir().unwrap_or_else(|_| PathBuf::from("."));
    path.push("config.toml");
    path
}

pub fn load_config(app: &AppHandle) -> AppConfig {
    let path = get_config_path(app);
    if path.exists() {
        if let Ok(content) = fs::read_to_string(&path) {
            if let Ok(config) = toml::from_str::<AppConfig>(&content) {
                return config;
            }
        }
    }
    let config = AppConfig::default();
    let _ = save_config(&config, app);
    config
}

pub fn save_config(config: &AppConfig, app: &AppHandle) -> std::io::Result<()> {
    let content = toml::to_string(config).map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
    let path = get_config_path(app);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, content)
}
