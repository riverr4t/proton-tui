use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
pub struct AppConfig {
    pub group_by_entry_ip: bool,
    #[serde(default)]
    pub split_view: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            group_by_entry_ip: true,
            split_view: false,
        }
    }
}

impl AppConfig {
    fn get_config_path() -> Result<PathBuf> {
        let mut path =
            dirs::config_dir().ok_or_else(|| anyhow::anyhow!("Could not find config directory"))?;
        path.push("proton-tui");
        fs::create_dir_all(&path)?;
        path.push("config.toml");
        Ok(path)
    }

    pub fn load() -> Result<Self> {
        let path = Self::get_config_path()?;
        if path.exists() {
            let content = fs::read_to_string(path)?;
            let config = toml::from_str(&content)?;
            Ok(config)
        } else {
            Ok(Self::default())
        }
    }

    pub fn save(&self) -> Result<()> {
        let path = Self::get_config_path()?;
        let content = toml::to_string_pretty(self)?;
        fs::write(path, content)?;
        Ok(())
    }
}
