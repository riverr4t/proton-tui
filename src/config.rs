use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

fn default_theme() -> String {
    "default".to_string()
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AppConfig {
    pub group_by_entry_ip: bool,
    #[serde(default)]
    pub split_view: bool,
    #[serde(default = "default_theme")]
    pub theme: String,
    #[serde(default)]
    pub favorites: Vec<String>,
    #[serde(default)]
    pub auto_connect: Option<String>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            group_by_entry_ip: true,
            split_view: false,
            theme: default_theme(),
            favorites: Vec::new(),
            auto_connect: None,
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
