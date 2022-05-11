use std::{env, fs, io};

use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct ConfigFile {
    editor: Option<String>,
    note_path: String,
    sync_command: String,
}

#[derive(Debug)]
pub struct Config {
    pub editor: String,
    pub note_path: String,
    pub sync_command: String,
}

#[derive(thiserror::Error, Debug)]
pub enum ConfigError {
    #[error("Could not find configuration directory")]
    GetDirectory,
    #[error("Error reading configuration file: {0}")]
    ReadFile(#[from] io::Error),
    #[error("Error in configuration file: {0}")]
    WrongConfig(#[from] toml::de::Error),
    #[error("Couldn't find editor in config and EDITOR variable is not set")]
    MissingEditor,
}

impl Config {
    pub fn read() -> Result<Self, ConfigError> {
        let mut config_dir = dirs::config_dir().ok_or_else(|| ConfigError::GetDirectory)?;

        config_dir.push("note");
        config_dir.push("config.toml");

        let file = fs::read_to_string(config_dir).map_err(|err| ConfigError::from(err))?;

        let config: ConfigFile = toml::from_str(&file).map_err(|err| ConfigError::from(err))?;

        let editor = match config.editor {
            Some(editor) => editor,
            None => env::var("EDITOR").map_err(|_| ConfigError::MissingEditor)?,
        };

        Ok(Self {
            editor,
            note_path: config.note_path,
            sync_command: config.sync_command,
        })
    }
}
