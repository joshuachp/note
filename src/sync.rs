use std::process::Command;

use color_eyre::{
    eyre::{ensure, Context},
    Result,
};

use crate::config::Config;

/// Execute sync file command
///
/// # Errors
///
/// This function will return an error if the command file.
pub fn execute_command(config: &Config) -> Result<()> {
    let res = Command::new(&config.sync_command)
        .spawn()
        .context("failed to spawn command")?
        .wait()
        .context("failed to wait cor command")?;

    ensure!(res.success(), "command exited with status {res}");

    Ok(())
}
