mod error;
mod parser;

use std::{ffi::OsStr, fs};

use indexmap::IndexMap;
use log::trace;
use walkdir::WalkDir;

use crate::error::Error;
use crate::parser::{parse, Markdown};

#[derive(Debug)]
struct File {
    file: String,
    content: String,
}

/// Convert markdown file to JSON
///
/// # Errors
///
/// - IO errors
/// - Missing headers
pub fn md_to_json(path: &str, skip_drafts: bool) -> Result<String, Error> {
    trace!("{}", path);

    let mut files: Vec<File> = Vec::new();

    WalkDir::new(path)
        .into_iter()
        .filter_map(Result::ok)
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

                trace!("{:?}", path);

                let content = fs::read_to_string(path).map_err(Error::File)?;
                let file = path
                    .file_stem()
                    .ok_or_else(|| Error::InvalidPath(path.to_string_lossy().to_string()))?
                    .to_string_lossy()
                    .to_string();

                trace!("{}", file);
                trace!("{}", content);

                files.push(File { file, content });

                Ok(files)
            },
        )
        .expect("Error reading files");

    trace!("{:?}", files);

    let mut markdown_files: IndexMap<&str, Markdown> = IndexMap::new();

    files
        .iter()
        .try_fold(
            &mut markdown_files,
            |markdown_files, file_content| -> Result<&mut IndexMap<&str, Markdown>, Error> {
                let File { file, content } = file_content;

                let markdown = parse(content)?;

                // Skip drafts
                if skip_drafts && markdown.draft {
                    return Ok(markdown_files);
                }

                markdown_files.insert(file, markdown);

                Ok(markdown_files)
            },
        )
        .expect("Error parsing markdown");

    trace!("{:?}", markdown_files);

    markdown_files.sort_by(|_, first, _, second| first.date.cmp(&second.date).reverse());

    trace!("sorted: {:?}", markdown_files);

    serde_json::to_string(&markdown_files).map_err(Error::ToJson)
}
