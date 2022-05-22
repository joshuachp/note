mod error;
mod parser;

use std::{collections::HashMap, ffi::OsStr, fs};

use log::trace;
use walkdir::WalkDir;

use crate::error::Error;
use crate::parser::{parse, Markdown};

pub fn md_to_json(path: &str) -> Result<String, Error> {
    trace!("{}", path);

    let mut files: HashMap<String, String> = HashMap::new();

    WalkDir::new(path)
        .into_iter()
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            trace!("{:?}", &entry);

            let path = entry.path();

            // Filter for markdown files
            path.is_file() && (path.extension() == Some(OsStr::new("md")))
        })
        .try_fold(
            &mut files,
            |files, entry| -> Result<&mut HashMap<String, String>, Error> {
                let path = entry.path();

                let content = fs::read_to_string(&path).map_err(Error::File)?;
                let file = path.file_stem().unwrap().to_string_lossy().to_string();

                trace!("{}", file);
                trace!("{}", content);

                files.insert(file, content);

                Ok(files)
            },
        )
        .expect("Error reading files");

    let mut markdown_files: HashMap<&str, Markdown> = HashMap::new();

    files
        .iter()
        .try_fold(
            &mut markdown_files,
            |markdown_files, (file, content)| -> Result<&mut HashMap<&str, Markdown>, Error> {
                markdown_files.insert(file, parse(content)?);
                Ok(markdown_files)
            },
        )
        .expect("Error parsing markdown");

    serde_json::to_string(&markdown_files).map_err(Error::ToJson)
}
