use std::{io, process::Command};

use crate::config::Config;

#[derive(Debug, thiserror::Error)]
pub enum SyncError {
    #[error("error executing sync command: {0}")]
    Spawn(io::Error),
    #[error("error waiting command: {0}")]
    Wait(io::Error),
    #[error("command did not finish successfully")]
    Result,
}

pub fn sync_files(config: &Config) -> Result<(), SyncError> {
    let res = Command::new(&config.sync_command)
        .spawn()
        .map_err(|err| SyncError::Spawn(err))?
        .wait()
        .map_err(|err| SyncError::Wait(err))?;

    if !res.success() {
        return Err(SyncError::Result);
    }

    Ok(())
}
