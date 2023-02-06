use chrono::{Local, NaiveDate};
use log::trace;
use std::{
    fs, io,
    path::PathBuf,
    process::{Command, ExitStatus},
    str::FromStr,
};

use crate::config::Config;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("error creating parent directory: {0}")]
    ParentDir(io::Error),
    #[error("path is not a file")]
    NotFile,
    #[error("failed to spawn editor: {0}")]
    Spawn(io::Error),
    #[error("failed to wait editor process: {0}")]
    Wait(io::Error),
    #[error("invalid date: {0}")]
    Date(chrono::ParseError),
    #[error("EDITOR commmand returned with error code {0}")]
    EditFailed(ExitStatus),
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

pub fn note(config: &Config, path: &str) -> Result<(), Error> {
    trace!("Path {}", path);

    let file_path = get_path(&config.note_path, path);

    // TODO: if parent doesn't exist create a temp file and then move it if file exists after edit
    if let Some(parent) = file_path.parent() {
        trace!("Parent {:?}", parent);

        if !parent.is_dir() {
            trace!("Creating parent directory: {parent:?}");

            fs::create_dir_all(parent).map_err(Error::ParentDir)?;
        }
    }

    match fs::metadata(&file_path) {
        Ok(metadata) => {
            trace!("File exists");

            if !metadata.is_file() {
                return Err(Error::NotFile);
            }
        }
        Err(err) => {
            trace!("Metadata error: {}", err);
        }
    }

    let res = Command::new(&config.editor)
        .args([&file_path])
        .spawn()
        .map_err(Error::Spawn)?
        .wait()
        .map_err(Error::Wait)?;

    trace!("Result {}", res);

    if !res.success() {
        return Err(Error::EditFailed(res));
    }

    Ok(())
}

pub fn journal(config: &Config, date: Option<&str>) -> Result<(), Error> {
    let date = match date {
        Some(date) => NaiveDate::from_str(date).map_err(Error::Date)?,
        None => Local::now().date_naive(),
    };

    let mut entry = PathBuf::from("journal");
    entry.push(date.to_string());
    entry.set_extension("md");

    note(config, &entry.to_string_lossy())
}
