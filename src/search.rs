use std::process::{Command, Stdio};

use color_eyre::{
    eyre::{ensure, Context},
    Result,
};
use tracing::{debug, trace};

use crate::{config::Config, edit::note};

pub fn execute_command(config: &Config, cmd: &str, search: &str) -> Result<Option<String>> {
    let output = Command::new(&config.shell)
        .current_dir(&config.note_path)
        .env("SEARCH", search)
        .args(["-c", cmd])
        .stdout(Stdio::piped())
        .spawn()
        .context("failed to execute search command")?
        .wait_with_output()
        .context("failed to execute search command")?;

    debug!("status: {}", &output.status);
    debug!("stdout: {:?}", &output.stdout);
    debug!("stderr: {:?}", &output.stderr);

    ensure!(
        output.status.success(),
        "command returned with status {}",
        output.status
    );

    let path = String::from_utf8(output.stdout).context("invalid UTF-8 in command output")?;

    if path.trim().is_empty() {
        debug!("empty path");

        return Ok(None);
    }

    Ok(Some(path))
}

pub fn find_file(config: &Config, file: &str) -> Result<()> {
    let Some(output) = execute_command(config, &config.find_command, file)? else {
        return Ok(());
    };

    trace!("{}", output);

    note(config, &output)
}

pub fn grep_content(config: &Config, search: &str) -> Result<()> {
    let Some(output) = execute_command(config, &config.search_command, search)? else {
        return Ok(());
    };

    trace!("{}", output);

    note(config, &output)
}
