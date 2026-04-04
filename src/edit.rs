use std::path::{Path, PathBuf};
use std::process::Command;
use std::str::FromStr;
use std::{fs, io};

use askama::Template;
use chrono::{Local, NaiveDate};
use color_eyre::Result;
use eyre::{Context, ensure};

use tracing::{debug, instrument, trace, warn};

use crate::config::Config;

#[derive(Debug, Template)]
#[template(path = "note.md", print = "code")]
struct Note {
    title: String,
    description: Option<String>,
    created_at: String,
    updated_at: Option<String>,
    lang: Option<String>,
    tags: Vec<String>,
}

#[instrument]
fn get_path(base_path: &Path, path: &str) -> PathBuf {
    let mut file_path = PathBuf::from(base_path);

    let path_string: String = path
        .chars()
        .map(|chr| {
            if chr.is_whitespace() {
                return '_';
            } else {
                chr.to_ascii_lowercase()
            }
        })
        .collect();

    trace!("{}", path_string);

    file_path.push(path_string);
    file_path.set_extension("md");

    trace!("{:?}", file_path);

    file_path
}

#[instrument(skip(config))]
pub fn note(config: &Config, path: &str) -> Result<()> {
    let path = path.trim();

    trace!(path);

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

#[instrument(skip(config))]
pub fn journal(config: &Config, base: &str, date: Option<&str>) -> Result<()> {
    let date = match date {
        Some(date) => {
            NaiveDate::from_str(date).wrap_err_with(|| format!("failed to parse date {date}"))?
        }
        None => Local::now().date_naive(),
    };

    let mut entry = PathBuf::from(base);
    entry.push(date.to_string());
    entry.set_extension("md");

    note(config, &entry.to_string_lossy())
}
