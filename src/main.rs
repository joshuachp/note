mod cli;
mod config;
mod edit;

use clap::Parser;
use config::Config;
use log::{error, trace};

use crate::{
    cli::{generate_completion, Cli, Command},
    edit::edit_note,
};

fn main() {
    env_logger::init();

    let cli = Cli::parse();

    trace!("{:?}", cli);

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
            Command::Journal => todo!(),
            Command::Todo => {
                if let Err(err) = edit_note(&config, "todo") {
                    error!("Error: {}", err);
                    panic!();
                }
            }
            Command::Search { content: _ } => todo!(),
            Command::Find { filename: _ } => todo!(),
            Command::Sync => todo!(),
            Command::Completion { shell } => generate_completion(shell),
        },
        None => {
            if let Err(err) = edit_note(&config, "inbox") {
                error!("Error: {}", err);
                panic!();
            }
        }
    }
}
