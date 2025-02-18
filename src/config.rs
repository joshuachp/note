use std::{env, fs, path::PathBuf};

use color_eyre::eyre::{Context, OptionExt};
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
    pub note_path: PathBuf,
    pub sync_command: String,
    pub find_command: String,
    pub search_command: String,
}

impl Config {
    pub fn read() -> color_eyre::Result<Self> {
        let mut config_dir =
            dirs::config_dir().ok_or_eyre("could not find configuration directory")?;

        config_dir.push("note");
        config_dir.push("config.toml");

        let file = fs::read_to_string(config_dir).wrap_err("reading configuration file")?;

        let config: ConfigFile = toml::from_str(&file).wrap_err("invalid configuration file")?;

        let shell = match config.shell {
            Some(shell) => shell,
            None => env::var("SHELL").wrap_err("failed to read SHELL environment variable")?,
        };

        let editor = match config.editor {
            Some(editor) => editor,
            None => env::var("EDITOR").wrap_err("failed to read EDITOR environment variable")?,
        };

        let note_path = match config.note_path {
            Some(note_path) => note_path,
            None => {
                env::var("NOTE_PATH").wrap_err("failed to read NOTE_PATH environment variable")?
            }
        };
        let note_path = PathBuf::from(note_path);
        let note_path = note_path
            .canonicalize()
            .wrap_err_with(|| format!("couldnt canonicalize path {}", note_path.display()))?;

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
