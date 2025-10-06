use std::{env, fs, io::ErrorKind, path::PathBuf, process::exit};

use serde::Deserialize;
use tracing::{error, info};

#[derive(Debug, Deserialize, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct RestAPIConfig {
    pub url: String,
    pub username: String,
    pub token: String,
}

#[derive(Debug, Deserialize, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub api: RestAPIConfig,
    pub rfid_devices: Vec<String>,
}

/// Config paths sorted by highest priority first
fn config_paths() -> Vec<PathBuf> {
    let mut paths: Vec<PathBuf> = Vec::new();
    let filename = "rustpolnak.toml";

    // 1. cli argument
    if let Some(path) = env::args().nth(1) {
        paths.push(path.into());
    }

    // 2. current directory
    paths.push(filename.into());

    // 3. $XDG_CONFIG_HOME
    if let Ok(xdg) = env::var("XDG_CONFIG_HOME") {
        paths.push(PathBuf::from(xdg).join(filename));
    }

    // 4. ~/.config
    if let Ok(home) = env::var("HOME") {
        paths.push(PathBuf::from(home).join(".config").join(filename));
    }

    paths
}

pub fn load_config() -> Config {
    for path in config_paths() {
        match fs::read_to_string(path.clone()) {
            Ok(toml_str) => match toml::from_str::<Config>(&toml_str) {
                Ok(config) => {
                    info!("Loaded config {path:?}");
                    return config;
                }
                Err(err) => {
                    error!("Failed to parse {path:?}: {err}");
                    exit(1);
                }
            },
            Err(err) => match err.kind() {
                ErrorKind::NotFound => info!("Config {path:?} not found"),
                _ => {
                    error!("Failed to load {path:?}: {err}");
                    exit(1);
                }
            },
        }
    }

    error!("No configuration file found!");
    exit(1);
}
