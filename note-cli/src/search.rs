use std::process::{Command, Stdio};

use color_eyre::{
    eyre::{ensure, Context},
    Result,
};
use log::{debug, trace};

use crate::{config::Config, edit::note};

pub fn execute_command(config: &Config, cmd: &str, search: &str) -> Result<String> {
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

    let result = String::from_utf8(output.stdout).context("failed to read command stdout")?;

    Ok(result)
}

pub fn find_file(config: &Config, file: &str) -> Result<()> {
    let output = execute_command(config, &config.find_command, file)?;

    trace!("{}", output);

    note(config, output.trim())?;
    Ok(())
}

pub fn grep_content(config: &Config, search: &str) -> Result<()> {
    let output = execute_command(config, &config.search_command, search)?;

    trace!("{}", output);

    note(config, output.trim())
}
