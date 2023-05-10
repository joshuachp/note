use std::{env, fs};

use color_eyre::{
    eyre::{Context, ContextCompat},
    Result,
};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct ConfigFile {
    shell: Option<String>,
    editor: Option<String>,
    #[serde(default)]
    change_dir: bool,
    note_path: Option<String>,
    sync_command: String,
    find_command: String,
    search_command: String,
}

impl Default for ConfigFile {
    fn default() -> Self {
        Self {
            shell: Default::default(),
            editor: Default::default(),
            change_dir: true,
            note_path: Default::default(),
            sync_command: String::new(),
            find_command: String::new(),
            search_command: String::new(),
        }
    }
}

#[derive(Debug, Default)]
pub struct Config {
    pub shell: String,
    pub editor: String,
    pub change_dir: bool,
    pub note_path: String,
    pub sync_command: String,
    pub find_command: String,
    pub search_command: String,
}

impl Config {
    pub fn read() -> Result<Self> {
        let mut config_dir =
            dirs::config_dir().context("could not find configuration directory")?;

        config_dir.push("note");
        config_dir.push("config.toml");

        let file = fs::read_to_string(config_dir).context("reading configuration file")?;

        let config: ConfigFile = toml::from_str(&file).context("invalid configuration file")?;

        let shell = match config.shell {
            Some(shell) => shell,
            None => env::var("SHELL").context("failed to read SHELL environment variable")?,
        };

        let editor = match config.editor {
            Some(editor) => editor,
            None => env::var("EDITOR").context("failed to read EDITOR environment variable")?,
        };

        let note_path = match config.note_path {
            Some(note_path) => note_path,
            None => {
                env::var("NOTE_PATH").context("failed to read NOTE_PATH environment variable")?
            }
        };

        Ok(Self {
            shell,
            editor,
            note_path,
            change_dir: config.change_dir,
            sync_command: config.sync_command,
            find_command: config.find_command,
            search_command: config.search_command,
        })
    }
}
