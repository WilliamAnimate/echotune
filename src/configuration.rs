#![cfg_attr(not(feature = "configuration"), allow(unused))]

#[cfg(feature = "configuration")]
use serde::Deserialize;

#[cfg(target_os = "linux")]
static DEFAULT_CFG_PATH: &'static str = ".config/echotune/echotune.toml";
#[cfg(target_os = "windows")]
static DEFAULT_CFG_PATH: &'static str = "AppData/Roaming/echotune/echotune.toml";
#[cfg(target_os = "macos")]
static DEFAULT_CFG_PATH: &'static str = "Library/Preferences/echotune/echotune.toml";

#[derive(Debug, Default)]
#[cfg_attr(feature = "configuration", derive(serde::Deserialize))]
pub struct Config {
    pub main: TomlMain,
    pub playlist: TomlPlaylist,
}

#[derive(Debug)]
#[cfg_attr(feature = "configuration", derive(serde::Deserialize), serde(default))]
pub struct TomlMain {
    pub crash_on_execute: bool,
}
impl Default for TomlMain {
    fn default() -> Self {
        Self {
            crash_on_execute: false,
        }
    }
}

#[derive(Debug)]
#[cfg_attr(feature = "configuration", derive(serde::Deserialize), serde(default))]
pub struct TomlPlaylist {
    pub never_use: bool,
    pub highlighted_color: String,
}
impl Default for TomlPlaylist {
    fn default() -> Self {
        Self {
            never_use: false,
            highlighted_color: "f5c2e7".to_string(),
        }
    }
}

impl Config {
    pub fn parse(to_parse: echotune::ConfigurationPath) -> Self {
    #[cfg(not(feature = "configuration"))] {
        return Config::default();
    }

    #[cfg(feature = "configuration")] {
        use std::fs::read_to_string;

        let file = match to_parse {
            echotune::ConfigurationPath::Default => DEFAULT_CFG_PATH,
            echotune::ConfigurationPath::Custom(s) => s
        };
        #[allow(deprecated)]
        let file = format!("{}/{}", std::env::home_dir().unwrap().to_string_lossy().to_string(), file);

        let buf = read_to_string(file).unwrap();

        let parsed: Config = basic_toml::from_str(&buf).unwrap();
        dbg!(&parsed);

        parsed
    }
    }
}

