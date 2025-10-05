use std::{fs::File, io::Read, path::Path};

use log::info;
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(default)]
pub struct Config {
    pub video_frames: u32,
    pub show_fps: bool,
    pub scale: u32,
    pub sound: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            video_frames: 2000,
            show_fps: false,
            scale: 4,
            sound: false,
        }
    }
}

impl Config {
    pub fn from_file<P: AsRef<Path>>(filepath: P) -> anyhow::Result<Config> {
        if !filepath.as_ref().try_exists()? {
            info!("Loaded default config");
            return Ok(Config::default());
        }

        let mut file = File::open(&filepath)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        let config: Config = toml::from_str(&contents)?;
        info!(
            "Loaded config from file {}",
            filepath.as_ref().to_str().unwrap()
        );
        Ok(config)
    }
}
