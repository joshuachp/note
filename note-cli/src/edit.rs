use chrono::{Local, NaiveDate};
use color_eyre::{
    eyre::{ensure, Context},
    Result,
};
use log::{debug, trace, warn};
use std::{
    fs, io,
    path::{Path, PathBuf},
    process::Command,
    str::FromStr,
};

use crate::config::Config;

fn get_path(base_path: &Path, path: &str) -> PathBuf {
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

pub fn note(config: &Config, path: &str) -> Result<()> {
    trace!("Path {}", path);

    let file_path = get_path(&config.note_path, path);

    // TODO: if parent doesn't exist create a temp file and then move it if file exists after edit
    if let Some(parent) = file_path.parent() {
        trace!("Parent {:?}", parent);

        if !parent.is_dir() {
            warn!("Creating parent directory: {parent:?}");

            fs::create_dir_all(parent).context("failed to create parent directories")?;
        }
    }

    // Ensure path, if it exists, it has to be a file.
    match fs::metadata(&file_path) {
        Ok(metadata) => ensure!(metadata.is_file(), "not a file"),
        Err(err) if err.kind() == io::ErrorKind::NotFound => {
            debug!("file '{file_path:?}' does not exists");
        }
        Err(err) => {
            return Err(err).with_context(|| format!("reading file metadata: {file_path:?}"));
        }
    }

    let mut command = Command::new(&config.editor);
    command.args([&file_path]);

    if config.change_dir {
        command.current_dir(&config.note_path);
    }

    let res = command
        .spawn()
        .context("failed to spawn editor")?
        .wait()
        .context("failed to wait to editor")?;

    trace!("Result {}", res);

    ensure!(res.success(), "editor returned with status code {res}");

    Ok(())
}

pub fn journal(config: &Config, date: Option<&str>) -> Result<()> {
    let date = match date {
        Some(date) => NaiveDate::from_str(date).context(format!("failed to parse date {date}"))?,
        None => Local::now().date_naive(),
    };

    let mut entry = PathBuf::from("journal");
    entry.push(date.to_string());
    entry.set_extension("md");

    note(config, &entry.to_string_lossy())
}
