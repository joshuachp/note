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

impl Config {
    pub fn read() -> io::Result<Self> {
        let mut config_dir = dirs::config_dir().expect("Couldn't find configuration directory");

        config_dir.push("note");
        config_dir.push("config.toml");

        let file = fs::read_to_string(config_dir).expect("Couldn't read file");

        let config: ConfigFile = toml::from_str(&file).expect("Error in config");

        let editor = match config.editor {
            Some(editor) => editor,
            None => env::var("EDITOR")
                .expect("Couldn't find editor in config and EDITOR variable is not set"),
        };

        Ok(Self {
            editor,
            note_path: config.note_path,
            sync_command: config.sync_command,
        })
    }
}
