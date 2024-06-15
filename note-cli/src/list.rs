use std::{
    fs,
    path::{Path, PathBuf},
};

use color_eyre::eyre::{Context, OptionExt};
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
    debug!("input path {:?} with max_depth {max_depth}", path);

    let path = get_path_to_list(config, path).wrap_err("couldn't get the path to list")?;

    debug!("chosen path {}", path.display());

    let entries = WalkDir::new(&path)
        .max_depth(max_depth)
        .follow_links(true)
        .sort_by(|a, b| {
            // Files first, then compare by name
            a.file_type()
                .is_file()
                .cmp(&b.file_type().is_file())
                // file (true) is greater than dir (false) so we reverse here
                .reverse()
                .then_with(|| a.file_name().cmp(b.file_name()))
        })
        .into_iter()
        .filter_entry(|e| {
            debug!("checking entry {}", e.path().display());

            !is_hidden(e)
        })
        .collect::<Result<Vec<DirEntry>, _>>()
        .wrap_err("couldn't read entry")?;

    for entry in entries {
        debug!("entry {}", entry.path().display());

        if entry.path() == path {
            debug!("skipping current");

            continue;
        }

        print_entry(entry, &config.note_path)?;
    }

    Ok(())
}

fn print_entry(entry: DirEntry, note_path: &Path) -> color_eyre::Result<()> {
    let metadata = entry
        .metadata()
        .wrap_err_with(|| format!("couldn't read metadata for {}", entry.path().display()))?;

    if metadata.is_dir() {
        let path = strip_note_prefix(note_path, entry.path())?;

        println!("{}/", path.display());

        return Ok(());
    }

    if !entry.path().extension().is_some_and(|ext| ext == "md") {
        debug!("ignoring non markdown file {}", entry.path().display());

        return Ok(());
    }

    let content = fs::read_to_string(entry.path())
        .wrap_err_with(|| format!("couldn't read file {}", entry.path().display()))?;

    let note = md_parser::parse(&content)
        .wrap_err_with(|| format!("couldn't parse {}", entry.path().display()))?;

    let path = strip_note_prefix(note_path, entry.path())?;

    println!("{}\t{}", path.display(), note.title);

    Ok(())
}

fn strip_note_prefix<'a>(note_path: &'a Path, path: &'a Path) -> color_eyre::Result<&'a Path> {
    if path.starts_with(note_path) {
        return path
            .strip_prefix(note_path)
            .wrap_err_with(|| format!("couldn't strip note path prefix from {}", path.display()));
    }

    Ok(path)
}

fn get_path_to_list(config: &Config, path: Option<PathBuf>) -> color_eyre::Result<PathBuf> {
    let Some(path) = path else {
        return Ok(config.note_path.clone());
    };

    if path.is_absolute() {
        return path
            .canonicalize()
            .wrap_err_with(|| format!("couldn't get absolute path {}", path.display()));
    }

    let local = path
        .try_exists()
        .wrap_err_with(|| format!("couldn't read relative path {}", path.display()))?;

    if local {
        return Ok(path.to_owned());
    }

    let mut note_path = config.note_path.join(&path);

    let exists = note_path
        .try_exists()
        .wrap_err_with(|| format!("couldn't read note path {}", note_path.display()))?;

    if exists {
        return Ok(note_path);
    }

    note_path
        .pop()
        .then_some(note_path)
        .ok_or_eyre("not a valid path")
}
