use std::{
    fs::{self},
    path::PathBuf,
};

use color_eyre::eyre::Context;
use log::debug;
use walkdir::{DirEntry, WalkDir};

use crate::config::Config;

fn is_hidden(entry: &DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| s.starts_with('.'))
        .unwrap_or(false)
}

pub fn list_path(
    config: &Config,
    path: Option<PathBuf>,
    max_depth: usize,
) -> color_eyre::Result<()> {
    let path = get_path_to_list(config, path).wrap_err("couldn't get the path to list")?;

    let entries = WalkDir::new(path)
        .max_depth(max_depth)
        .follow_links(true)
        .into_iter()
        .filter_entry(|e| !is_hidden(e))
        .collect::<Result<Vec<DirEntry>, _>>()
        .wrap_err("couldn't read entry")?;

    for entry in entries {
        print_entry(entry)?;
    }

    Ok(())
}

fn print_entry(entry: DirEntry) -> color_eyre::Result<()> {
    let metadata = entry
        .metadata()
        .wrap_err_with(|| format!("couldn't read metadata for {}", entry.path().display()))?;

    if metadata.is_dir() {
        println!("{}", entry.path().display());
    }

    if !entry.path().extension().is_some_and(|ext| ext == "md") {
        debug!("ignoring non markdown file {}", entry.path().display());

        return Ok(());
    }

    let content = fs::read_to_string(entry.path())
        .wrap_err_with(|| format!("couldn't read file {}", entry.path().display()))?;

    let note = md_parser::parse(&content)
        .wrap_err_with(|| format!("couldn't parse {}", entry.path().display()))?;

    println!("{}\t{}", entry.path().display(), note.title);

    Ok(())
}

fn get_path_to_list(config: &Config, path: Option<PathBuf>) -> color_eyre::Result<PathBuf> {
    if let Some(path) = path {
        return Ok(path);
    }

    let note_path = &config.note_path;

    let cwd = std::env::current_dir()
        .and_then(|dir| dir.canonicalize())
        .wrap_err("couldn't get CWD")?;

    if cwd.starts_with(note_path) {
        return Ok(cwd);
    }

    Ok(note_path.to_owned())
}
