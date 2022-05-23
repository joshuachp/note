mod error;
mod parser;

use std::{ffi::OsStr, fs};

use indexmap::IndexMap;
use log::trace;
use walkdir::WalkDir;

use crate::error::Error;
use crate::parser::{parse, Markdown};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
struct File {
    file: String,
    content: String,
}

pub fn md_to_json(path: &str) -> Result<String, Error> {
    trace!("{}", path);

    let mut files: Vec<File> = Vec::new();

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
            |files, entry| -> Result<&mut Vec<File>, Error> {
                let path = entry.path();

                let content = fs::read_to_string(&path).map_err(Error::File)?;
                let file = path.file_stem().unwrap().to_string_lossy().to_string();

                trace!("{}", file);
                trace!("{}", content);

                files.push(File { file, content });

                Ok(files)
            },
        )
        .expect("Error reading files");

    files.sort();
    trace!("{:?}", files);

    let mut markdown_files: IndexMap<&str, Markdown> = IndexMap::new();

    files
        .iter()
        .try_fold(
            &mut markdown_files,
            |markdown_files, file_content| -> Result<&mut IndexMap<&str, Markdown>, Error> {
                let File { file, content } = file_content;
                markdown_files.insert(file, parse(content)?);
                Ok(markdown_files)
            },
        )
        .expect("Error parsing markdown");

    serde_json::to_string(&markdown_files).map_err(Error::ToJson)
}
