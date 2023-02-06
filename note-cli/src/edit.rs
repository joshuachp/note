use chrono::{Local, NaiveDate};
use color_eyre::{
    eyre::{ensure, Context},
    Result,
};
use log::trace;
use std::{fs, path::PathBuf, process::Command, str::FromStr};

use crate::config::Config;

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

pub fn note(config: &Config, path: &str) -> Result<()> {
    trace!("Path {}", path);

    let file_path = get_path(&config.note_path, path);

    // TODO: if parent doesn't exist create a temp file and then move it if file exists after edit
    if let Some(parent) = file_path.parent() {
        trace!("Parent {:?}", parent);

        if !parent.is_dir() {
            trace!("Creating parent directory: {parent:?}");

            fs::create_dir_all(parent).context("failed to create parent directories")?;
        }
    }

    let metadata = fs::metadata(&file_path).context("reading file metadata")?;

    ensure!(metadata.is_file(), "not a file");

    let res = Command::new(&config.editor)
        .args([&file_path])
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
