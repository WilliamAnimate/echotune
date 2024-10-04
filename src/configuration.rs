use serde::Deserialize;

#[cfg(target_os = "linux")]
static DEFAULT_CFG_PATH: &'static str = ".config/echotune/echotune.toml";
#[cfg(target_os = "windows")]
static DEFAULT_CFG_PATH: &'static str = "AppData/Roaming/echotune/echotune.toml";
#[cfg(target_os = "macos")]
static DEFAULT_CFG_PATH: &'static str = "Library/Preferences/echotune/echotune.toml";

#[derive(Deserialize, Debug)]
pub struct Config {
    pub main: TomlMain
}

#[derive(Deserialize, Debug)]
pub struct TomlMain {
    pub crash_on_execute: bool,
}

impl Config {
    pub fn parse(to_parse: echotune::ConfigurationMode) -> Self {
        use std::fs::read_to_string;

        let file = match to_parse {
            echotune::ConfigurationMode::Default => DEFAULT_CFG_PATH,
            echotune::ConfigurationMode::Custom(s) => s
        };
        #[allow(deprecated)]
        let file = format!("{}/{}", std::env::home_dir().unwrap().to_string_lossy().to_string(), file);

        let buf = read_to_string(file).unwrap();

        let parsed: Config = basic_toml::from_str(&buf).unwrap();
        dbg!(&parsed);

        parsed
    }
}

