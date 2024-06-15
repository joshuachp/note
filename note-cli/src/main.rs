mod cli;
mod config;
mod edit;
mod list;
mod search;
mod sync;

use clap::Parser;
use color_eyre::eyre::Context;
use config::Config;
use log::debug;

use crate::{
    cli::{generate_completion, Cli, Command},
    edit::{journal, note},
    list::list_path,
    search::{find_file, grep_content},
    sync::execute_command,
};

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    env_logger::init();

    let cli = Cli::parse();

    debug!("{:?}", cli);

    // Call before reading the config
    if let Some(Command::Completion { shell }) = cli.command {
        generate_completion(shell)?;

        return Ok(());
    }

    let config = Config::read().wrap_err("couldn't read configuration")?;

    debug!("{:?}", config);

    match cli.command {
        Some(command) => match command {
            Command::Edit(edit) => note(&config, &edit.path),
            Command::Journal { date } => journal(&config, date.as_deref()),
            Command::Todo => note(&config, "todo"),
            Command::Search { content } => {
                let content = content.as_deref().unwrap_or("");

                grep_content(&config, content)?;

                Ok(())
            }
            Command::Find { filename } => {
                let content = filename.as_deref().unwrap_or("");
                find_file(&config, content)?;

                Ok(())
            }
            Command::Sync => execute_command(&config),
            Command::List { path, max_depth } => list_path(&config, path, max_depth),
            Command::Completion { .. } => unreachable!("already matched"),
        },
        None => note(&config, "inbox"),
    }
}
