use std::{env, fs, io};

use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct ConfigFile {
    shell: Option<String>,
    editor: Option<String>,
    note_path: Option<String>,
    sync_command: String,
    find_command: String,
    search_command: String,
}

#[derive(Debug)]
pub struct Config {
    pub shell: String,
    pub editor: String,
    pub note_path: String,
    pub sync_command: String,
    pub find_command: String,
    pub search_command: String,
}

#[derive(thiserror::Error, Debug)]
pub enum ConfigError {
    #[error("could not find configuration directory")]
    GetDirectory,
    #[error("error reading configuration file: {0}")]
    ReadFile(io::Error),
    #[error("invalid configuration file: {0}")]
    WrongConfig(toml::de::Error),
    #[error("no shell set in configuration and could not read SHELL env variable: {0}")]
    MissingShell(env::VarError),
    #[error("no editor set in configuration and could not read EDITOR env variable: {0}")]
    MissingEditor(env::VarError),
    #[error("no note path set in configuration and could not read NOTE_PATH env variable: {0}")]
    MissingNotePath(env::VarError),
}

impl Config {
    pub fn read() -> Result<Self, ConfigError> {
        let mut config_dir = dirs::config_dir().ok_or(ConfigError::GetDirectory)?;

        config_dir.push("note");
        config_dir.push("config.toml");

        let file = fs::read_to_string(config_dir).map_err(ConfigError::ReadFile)?;

        let config: ConfigFile = toml::from_str(&file).map_err(ConfigError::WrongConfig)?;

        let shell = match config.shell {
            Some(shell) => shell,
            None => env::var("SHELL").map_err(ConfigError::MissingShell)?,
        };

        let editor = match config.editor {
            Some(editor) => editor,
            None => env::var("EDITOR").map_err(ConfigError::MissingEditor)?,
        };

        let note_path = match config.note_path {
            Some(note_path) => note_path,
            None => env::var("NOTE_PATH").map_err(ConfigError::MissingNotePath)?,
        };

        Ok(Self {
            shell,
            editor,
            note_path,
            sync_command: config.sync_command,
            find_command: config.find_command,
            search_command: config.search_command,
        })
    }
}
