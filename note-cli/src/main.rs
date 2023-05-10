mod cli;
mod config;
mod edit;
mod search;
mod sync;

use clap::Parser;
use color_eyre::{Report, Result};
use config::Config;
use log::trace;
use md_json::md_to_json;

use crate::{
    cli::{generate_completion, Cli, Command},
    edit::{journal, note},
    search::{find_file, grep_content},
    sync::execute_command,
};

fn main() -> Result<(), Report> {
    color_eyre::install()?;
    env_logger::init();

    let cli = Cli::parse();

    trace!("{:?}", cli);

    let config = Config::read().unwrap_or_default();

    trace!("{:?}", config);

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
            Command::Compile { path, drafts } => {
                let json = md_to_json(path, !drafts)?;
                println!("{json}");

                Ok(())
            }
            Command::Completion { shell } => generate_completion(shell),
        },
        None => note(&config, "inbox"),
    }
}
