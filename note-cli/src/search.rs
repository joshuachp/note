use std::{
    io,
    process::{Command, Stdio},
};

use log::{debug, trace};

use crate::{
    config::Config,
    edit::{note, Error as EditError},
};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("failed to execute the command: {0}")]
    Exec(io::Error),
    #[error("command finished with errors: {0}")]
    Status(String),
    #[error("invalid UTF8 output: {0}")]
    Utf8(std::string::FromUtf8Error),
    #[error("failed to edit note: {0}")]
    Edit(EditError),
}

pub fn execute_command(config: &Config, cmd: &str, search: &str) -> Result<String, Error> {
    let output = Command::new(&config.shell)
        .current_dir(&config.note_path)
        .env("SEARCH", search)
        .args(["-c", cmd])
        .stdout(Stdio::piped())
        .spawn()
        .map_err(Error::Exec)?
        .wait_with_output()
        .map_err(Error::Exec)?;

    debug!("status: {}", &output.status);
    debug!("stdout: {:?}", &output.stdout);
    debug!("stderr: {:?}", &output.stderr);

    if !output.status.success() {
        let stderr = String::from_utf8(output.stderr).map_err(Error::Utf8)?;

        return Err(Error::Status(stderr));
    }

    let result = String::from_utf8(output.stdout).map_err(Error::Utf8)?;

    Ok(result)
}

pub fn find_file(config: &Config, file: &str) -> Result<(), Error> {
    let output = execute_command(config, &config.find_command, file)?;

    trace!("{}", output);

    note(config, output.trim()).map_err(Error::Edit)
}

pub fn grep_content(config: &Config, search: &str) -> Result<(), Error> {
    let output = execute_command(config, &config.search_command, search)?;

    trace!("{}", output);

    note(config, output.trim()).map_err(Error::Edit)
}
