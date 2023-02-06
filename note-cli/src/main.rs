mod cli;
mod config;
mod edit;
mod search;
mod sync;

use clap::Parser;
use config::Config;
use log::{error, trace};
use md_json::md_to_json;

use crate::{
    cli::{generate_completion, Cli, Command},
    edit::{journal, note},
    search::{find_file, grep_content},
    sync::execute_command,
};

fn main() {
    env_logger::init();

    let cli = Cli::parse();

    trace!("{:?}", cli);

    if let Some(Command::Completion { shell }) = cli.command {
        generate_completion(shell);
        return;
    }

    let config = match Config::read() {
        Ok(config) => config,
        Err(err) => {
            error!("Error: {}", err);
            panic!();
        }
    };

    trace!("{:?}", config);

    match cli.command {
        Some(command) => match command {
            Command::Edit(edit) => {
                if let Err(err) = note(&config, &edit.path) {
                    error!("Error: {}", err);
                    panic!();
                }
            }
            Command::Journal { date } => {
                if let Err(err) = journal(&config, date.as_deref()) {
                    error!("Error: {}", err);
                    panic!();
                }
            }
            Command::Todo => {
                if let Err(err) = note(&config, "todo") {
                    error!("Error: {}", err);
                    panic!();
                }
            }
            Command::Search { content } => {
                let content = content.as_deref().unwrap_or("");
                if let Err(err) = grep_content(&config, content) {
                    error!("Error: {}", err);
                    panic!();
                }
            }
            Command::Find { filename } => {
                let content = filename.as_deref().unwrap_or("");
                if let Err(err) = find_file(&config, content) {
                    error!("Error: {}", err);
                    panic!();
                }
            }
            Command::Sync => {
                if let Err(err) = execute_command(&config) {
                    error!("Error: {}", err);
                    panic!();
                }
            }
            Command::Compile { path, drafts } => match md_to_json(&path, !drafts) {
                Err(err) => {
                    error!("Error: {}", err);
                    panic!();
                }
                Ok(json) => println!("{json}"),
            },
            Command::Completion { .. } => unreachable!("should have returned before"),
        },
        None => {
            if let Err(err) = note(&config, "inbox") {
                error!("Error: {}", err);
                panic!();
            }
        }
    }
}
