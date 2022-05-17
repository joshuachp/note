mod error;
mod parser;

use std::{collections::HashMap, ffi::OsStr, fs};

use walkdir::WalkDir;

use crate::error::Error;
use crate::parser::{parse, Markdown};

pub fn md_to_json(path: &str) -> Result<String, Error> {
    let mut files: HashMap<String, String> = HashMap::new();

    WalkDir::new(path)
        .into_iter()
        .filter_map(|entry| entry.ok())
        .filter_map(|entry| {
            let path = entry.path();

            if path.is_file() && path.extension() == Some(OsStr::new("md")) {
                Some(path.to_string_lossy().to_string())
            } else {
                None
            }
        })
        .try_fold(
            &mut files,
            |files, file| -> Result<&mut HashMap<String, String>, Error> {
                let content = fs::read_to_string(&file).map_err(|err| Error::File(err))?;

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

    serde_json::to_string(&markdown_files).map_err(|err| Error::ToJson(err))
}
