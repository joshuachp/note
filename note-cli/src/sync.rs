use std::{io, process::Command};

use crate::config::Config;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("error executing sync command: {0}")]
    Spawn(io::Error),
    #[error("error waiting command: {0}")]
    Wait(io::Error),
    #[error("command did not finish successfully")]
    Result,
}

/// Execute sync file command
///
/// # Errors
///
/// This function will return an error if the command file.
pub fn execute_command(config: &Config) -> Result<(), Error> {
    let res = Command::new(&config.sync_command)
        .spawn()
        .map_err(Error::Spawn)?
        .wait()
        .map_err(Error::Wait)?;

    if !res.success() {
        return Err(Error::Result);
    }

    Ok(())
}
