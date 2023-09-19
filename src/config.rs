use std::fs;
use std::path::{Path, PathBuf};
use eyre::{eyre, Result};
use serde::Deserialize;
use toml;

#[derive(Debug, Deserialize,Clone)]
pub struct Generator {
    pub command: Option<String>,
    pub args: Option<Vec<String>>,
    pub patterns: Option<Vec<String>>,
}

#[derive(Debug, Deserialize,Clone)]
pub struct Config {
    pub generators: Vec<Generator>,
}

pub fn read_config(config_path: Option<PathBuf>) -> Result<(Config, PathBuf)> {
    let config_path = if let Some(path) = config_path {
        path
    } else {
        // Automatically locate the config file in the root of the repository
        let mut current_dir = Path::new(".").canonicalize()?;
        loop {
            let config_file = current_dir.join("nomgen.toml");
            if config_file.is_file() {
                break config_file;
            }
            if !current_dir.pop() {
                return Err(eyre!("Failed to locate config file."));
            }
        }
    };

    let config_str = fs::read_to_string(&config_path)?;
    let config: Config = toml::from_str(&config_str)?;

    if config.generators.is_empty() {
        return Err(eyre!("No generators found in the configuration."));
    }

    Ok((config, config_path))
}
