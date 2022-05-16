use std::{
    io,
    process::{Command, Stdio},
};

use log::debug;

use crate::config::Config;

#[derive(Debug, thiserror::Error)]
pub enum SearchError {
    #[error("failed to execute the command: {0}")]
    Exec(io::Error),
    #[error("command finished with errors: {0}")]
    Status(String),
    #[error("invalid UTF8 output: {0}")]
    Utf8(std::string::FromUtf8Error),
}

pub fn shell_cmd(config: &Config, cmd: &str, search: &str) -> Result<String, SearchError> {
    let output = Command::new(&config.shell)
        .current_dir(&config.note_path)
        .env("search", search)
        .args(["-c", cmd])
        .stdout(Stdio::piped())
        .spawn()
        .map_err(|err| SearchError::Exec(err))?
        .wait_with_output()
        .map_err(|err| SearchError::Exec(err))?;

    debug!("status: {}", &output.status);
    debug!("stdout: {:?}", &output.stdout);
    debug!("stderr: {:?}", &output.stderr);

    if !output.status.success() {
        let stderr = String::from_utf8(output.stderr).map_err(|err| SearchError::Utf8(err))?;

        return Err(SearchError::Status(stderr));
    }

    let result = String::from_utf8(output.stdout).map_err(|err| SearchError::Utf8(err))?;

    Ok(result)
}

pub fn find_file(config: &Config, file: &str) -> Result<(), SearchError> {
    let output = shell_cmd(config, &config.find_command, file)?;

    dbg!(output);

    Ok(())
}

pub fn search_content(config: &Config, search: &str) -> Result<(), SearchError> {
    let output = shell_cmd(config, &config.search_command, search)?;

    dbg!(output);

    Ok(())
}
