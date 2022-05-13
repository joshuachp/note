use chrono::{Local, NaiveDate};
use log::trace;
use std::{fs, io, path::PathBuf, process::Command, str::FromStr};

use crate::config::Config;

#[derive(Debug, thiserror::Error)]
pub enum EditError {
    #[error("error creating parent directory: {0}")]
    ParentDir(io::Error),
    #[error("path is not a file")]
    NotFile,
    #[error("failed to spawn editor")]
    Spawn(io::Error),
    #[error("failed to wait editor process")]
    Wait(io::Error),
    #[error("invalid date: {0}")]
    Date(chrono::ParseError),
}

fn get_path(base_path: &str, path: &str) -> PathBuf {
    let mut file_path = PathBuf::from(base_path);

    let path_string: String = path
        .chars()
        .map(|chr| {
            if chr.is_whitespace() {
                return '_';
            }
            chr.to_ascii_lowercase()
        })
        .collect();

    trace!("{}", path_string);

    file_path.push(path_string);
    file_path.set_extension("md");

    trace!("{:?}", file_path);

    file_path
}

pub fn edit_note(config: &Config, path: &str) -> Result<(), EditError> {
    trace!("Path {}", path);

    let file_path = get_path(&config.note_path, path);

    // TODO: if parent doesn't exist create a temp file and then move it if file exists after edit
    if let Some(parent) = file_path.parent() {
        trace!("Parent {:?}", parent);

        if !parent.is_dir() {
            trace!("Creating parent directory: {parent:?}");

            fs::create_dir_all(parent).map_err(|err| EditError::ParentDir(err))?;
        }
    }

    match fs::metadata(&file_path) {
        Ok(metadata) => {
            trace!("File exists");

            if !metadata.is_file() {
                return Err(EditError::NotFile);
            }
        }
        Err(err) => {
            trace!("Metadata error: {}", err);
        }
    }

    let res = Command::new(&config.editor)
        .args([&file_path])
        .spawn()
        .map_err(|err| EditError::Spawn(err))?
        .wait()
        .map_err(|err| EditError::Wait(err))?;

    trace!("Result {}", res);

    if !res.success() {
        panic!("Command fail");
    }

    Ok(())
}

pub fn edit_journal(config: &Config, date: Option<&str>) -> Result<(), EditError> {
    let date = match date {
        Some(date) => NaiveDate::from_str(date).map_err(|err| EditError::Date(err))?,
        None => Local::today().naive_local(),
    };

    let mut entry = PathBuf::from("journal");
    entry.push(date.to_string());
    entry.set_extension("md");

    edit_note(config, &entry.to_string_lossy())
}
