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
    edit::{edit_journal, edit_note},
    search::{find_file, search_content},
    sync::sync_files,
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
                if let Err(err) = edit_note(&config, &edit.path) {
                    error!("Error: {}", err);
                    panic!();
                }
            }
            Command::Journal { date } => {
                if let Err(err) = edit_journal(&config, date.as_deref()) {
                    error!("Error: {}", err);
                    panic!();
                }
            }
            Command::Todo => {
                if let Err(err) = edit_note(&config, "todo") {
                    error!("Error: {}", err);
                    panic!();
                }
            }
            Command::Search { content } => {
                let content = content.as_deref().unwrap_or_else(|| "");
                if let Err(err) = search_content(&config, content) {
                    error!("Error: {}", err);
                    panic!();
                }
            }
            Command::Find { filename } => {
                let content = filename.as_deref().unwrap_or_else(|| "");
                if let Err(err) = find_file(&config, content) {
                    error!("Error: {}", err);
                    panic!();
                }
            }
            Command::Sync => {
                if let Err(err) = sync_files(&config) {
                    error!("Error: {}", err);
                    panic!();
                }
            }
            Command::Compile { path } => match md_to_json(&path) {
                Err(err) => {
                    error!("Error: {}", err);
                    panic!();
                }
                Ok(json) => println!("{}", json),
            },
            Command::Completion { .. } => unreachable!("should have returned before"),
        },
        None => {
            if let Err(err) = edit_note(&config, "inbox") {
                error!("Error: {}", err);
                panic!();
            }
        }
    }
}
