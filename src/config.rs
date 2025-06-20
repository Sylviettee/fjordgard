use std::fs;

use directories::ProjectDirs;
use serde::{Deserialize, Serialize};

#[derive(
    Serialize, Deserialize, Debug, Clone, Copy, PartialEq, strum::Display, strum::VariantArray,
)]
pub enum BackgroundMode {
    Unsplash,
    Solid,
    Local,
}

impl BackgroundMode {
    pub fn default_background(&self) -> &'static str {
        match self {
            // https://unsplash.com/collections/1053828/tabliss-official
            Self::Unsplash => "1053828",
            Self::Solid => "#000000",
            Self::Local => "",
        }
    }

    pub fn edit_text(&self) -> &'static str {
        match self {
            Self::Unsplash => "Unsplash collection",
            Self::Solid => "Color (#rrggbb)",
            Self::Local => "File path",
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Location {
    pub longitude: f64,
    pub latitude: f64,
    pub name: Option<String>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Config {
    pub time_format: String,
    pub background_mode: BackgroundMode,
    pub background: String,
    pub unsplash_key: Option<String>,
    pub location: Option<Location>,
}

impl Config {
    pub fn load() -> anyhow::Result<Config> {
        if let Some(dir) = ProjectDirs::from("gay.gayest", "", "fjordgard") {
            let config_file = dir.config_dir().join("config.json");

            if !config_file.exists() {
                return Ok(Config::default());
            }

            let data = fs::read_to_string(config_file)?;

            Ok(serde_json::from_str(&data)?)
        } else {
            Ok(Config::default())
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub async fn save(&self) -> anyhow::Result<()> {
        if let Some(dir) = ProjectDirs::from("gay.gayest", "", "fjordgard") {
            let config_dir = dir.config_dir();
            tokio::fs::create_dir_all(config_dir).await?;

            let contents = serde_json::to_string(self)?;

            tokio::fs::write(config_dir.join("config.json"), contents).await?;

            Ok(())
        } else {
            anyhow::bail!("no config directory found")
        }
    }

    #[cfg(target_arch = "wasm32")]
    pub async fn save(&self) -> anyhow::Result<()> {
        Ok(())
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            time_format: String::from("%-I:%M:%S"),
            background_mode: BackgroundMode::Solid,
            background: BackgroundMode::Solid.default_background().to_string(),
            unsplash_key: None,
            location: None,
        }
    }
}
