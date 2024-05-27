mod error;
mod parser;

use std::fmt::Debug;
use std::fs;
use std::path::{Path, PathBuf};

use indexmap::IndexMap;
use tracing::{instrument, trace};
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
#[instrument]
pub fn md_to_json<P>(path: P, skip_drafts: bool) -> Result<String, Error>
where
    P: AsRef<Path> + Debug,
{
    let path = path.as_ref();

    trace!("{}", path.display());

    if !PathBuf::from(path).exists() {
        return Err(Error::InvalidPath(path.to_path_buf()));
    }

    let files = WalkDir::new(path)
        .into_iter()
        .filter_map(Result::ok)
        .filter_map(|entry| {
            let path = entry.into_path();

            // Filter for markdown files
            match path.extension() {
                Some(md) if md == "md" && path.is_file() => Some(path),
                Some(_) | None => None,
            }
        })
        .map(|path| {
            // NOTE: maybe we have to canonicalize the path
            trace!("{}", path.display());

            let content = fs::read_to_string(&path).map_err(Error::File)?;
            let file = path
                .file_stem()
                .and_then(|s| s.to_str())
                .map(|s| s.to_string())
                .ok_or_else(|| Error::InvalidPath(path))?;

            trace!("file: `{}` content: `{}`", file, content);

            Ok(File { file, content })
        })
        .collect::<Result<Vec<File>, Error>>()?;

    trace!("{:?}", files);

    let mut markdown_files = files
        .iter()
        .filter_map(|file_content| {
            let File { file, content } = file_content;

            match parse(content) {
                Ok(markdown) if skip_drafts && !markdown.released => None,
                Ok(markdown) => Some(Ok((file.as_str(), markdown))),
                Err(err) => {
                    trace!("error: {:?}", err);

                    Some(Err(err))
                }
            }
        })
        .collect::<Result<IndexMap<&str, Markdown>, Error>>()?;

    trace!("{:?}", markdown_files);

    markdown_files.sort_by(|_, first, _, second| first.updated.cmp(&second.updated).reverse());

    trace!("sorted: {:?}", markdown_files);

    // serde_json::to_string(&markdown_files).map_err(Error::ToJson)
    Ok(format!("{markdown_files:?}"))
}

#[cfg(test)]
mod test {
    use tempfile::TempDir;

    const EXAMPLE_MD: &str = include_str!("../assets/example.md");

    use super::*;

    #[test]
    pub fn should_walk_dirs() {
        let dir = TempDir::with_prefix("md-json").expect("failed to create temp dir");

        let file = dir.path().join("test.md");

        std::fs::write(file, EXAMPLE_MD).expect("failed to write file");

        let json = md_to_json(dir.path(), false);

        assert!(
            json.is_ok(),
            "failed to convert markdown to json {}",
            json.unwrap_err()
        );
    }
}
