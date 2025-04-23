use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;

use crate::app::Error;

const APP_NAME: &str = "aristotle";
const CONF_FILE: &str = "config.toml";

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub library_path: PathBuf,
    pub family: String,
    pub font_size: f32,
    pub horizontal_margin: u8,
    pub vertical_margin: u8,
    pub page_width: usize,
    pub page_height: usize,
}
impl Config {
    pub fn load_config() -> Result<Self, Error> {
        let dir = dirs::config_dir().unwrap().join(APP_NAME);
        let path = dir.join(CONF_FILE);
        let config = if path.exists() {
            let content = fs::read_to_string(&path)?;
            toml::from_str(&content)?
        } else {
            tracing::warn!("config file not found, using defaults");
            let config = Self::defaults();
            let _ = std::fs::create_dir(dir);
            let mut file = File::create(path)?;
            let contents = toml::to_string(&config)?;
            file.write_all(contents.as_ref())?;
            config
        };
        Ok(config)
    }

    fn defaults() -> Self {
        let lib = dirs::data_dir().unwrap().join(APP_NAME);
        Self {
            library_path: lib,
            family: "Vollkorn".to_owned(),
            font_size: 18.0,
            horizontal_margin: 16,
            vertical_margin: 16,
            page_width: 600,
            page_height: 800,
        }
    }
}
