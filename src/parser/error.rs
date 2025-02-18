use std::{io, path::PathBuf};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("impossible to read the file")]
    File(io::Error),
    #[error("failed to serialize to json")]
    ToJson(serde_json::Error),
    #[error("invalid date format")]
    Date(chrono::format::ParseError),
    #[error("invalid path {0}")]
    InvalidPath(PathBuf),
    #[error("missing or incomplete frontmatter")]
    MissingFrontmatter,
    #[error("couldn't parse front-mattter: {0}")]
    FrontMatter(winnow::error::ContextError),
}
