use std::io;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("invalid yaml front matter: {0}")]
    FrontMatter(serde_yaml::Error),
    #[error("impossible to read the file: {0}")]
    File(io::Error),
    #[error("failed to serialize to json: {0}")]
    ToJson(serde_json::Error),
}
