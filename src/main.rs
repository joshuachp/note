mod cli;
mod config;
mod edit;

use clap::Parser;
use config::Config;
use log::{error, trace};

use crate::{
    cli::{Cli, Command},
    edit::edit_note,
};

fn main() {
    env_logger::init();

    let cli = Cli::parse();

    trace!("{:?}", cli);

    let config = match Config::read() {
        Ok(config) => config,
        Err(err) => {
            error!("{}", err);
            panic!();
        }
    };

    trace!("{:?}", config);

    match cli.command {
        Command::Edit { path } | Command::E { path } => edit_note(&config, &path),
        Command::Journal => todo!(),
        Command::Todo => edit_note(&config, "todo"),
        Command::Search { content: _ } => todo!(),
        Command::Find { filename: _ } => todo!(),
        Command::Sync => todo!(),
        Command::Completion { shell: _ } => todo!(),
    }
}
