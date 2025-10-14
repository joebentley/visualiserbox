use raylib::core::color as rl_color;
use std::{fs::File, io::Read, path::Path};

use log::info;
use serde::Deserialize;

fn two_five_five() -> u8 {
    255
}

#[derive(Copy, Clone, Debug, Deserialize)]
#[serde(default)]
pub struct Colour {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    #[serde(default = "two_five_five")]
    pub a: u8,
}

impl From<rl_color::Color> for Colour {
    fn from(value: rl_color::Color) -> Colour {
        Colour {
            r: value.r,
            g: value.g,
            b: value.b,
            a: value.a,
        }
    }
}

impl From<Colour> for rl_color::Color {
    fn from(value: Colour) -> rl_color::Color {
        rl_color::Color {
            r: value.r,
            g: value.g,
            b: value.b,
            a: value.a,
        }
    }
}

impl Default for Colour {
    fn default() -> Self {
        rl_color::Color::NAVAJOWHITE.into()
    }
}

#[derive(Copy, Clone, Debug, Deserialize)]
#[serde(default)]
pub struct Config {
    pub video_frames: u32,
    pub show_fps: bool,
    pub scale: u32,
    pub sound: bool,
    pub sequence_speed: f32,
    pub pause_time: f32,
    pub primary_colour: Colour,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            video_frames: 2000,
            show_fps: false,
            scale: 4,
            sound: false,
            sequence_speed: 0.001,
            pause_time: 0.5,
            primary_colour: Default::default(),
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

        info!("{:?}", config);

        Ok(config)
    }
}
