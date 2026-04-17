use std::io::IsTerminal;

use clap::Parser;
use config::Config;
use eyre::WrapErr;
use tracing::{debug, trace};
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

use crate::{
    cli::{Cli, Command},
    edit::{journal, note},
    list::list_path,
    query::query,
    search::{find_file, grep_content},
};

use self::edit::work;

mod cli;
mod config;
mod edit;
mod list;
mod query;
mod search;

fn main() -> eyre::Result<()> {
    let cli = Cli::parse();

    color_eyre::install()?;

    tracing_subscriber::registry()
        .with(EnvFilter::from_default_env())
        .with(
            tracing_subscriber::fmt::layer()
                .with_writer(std::io::stderr)
                .with_ansi(std::io::stderr().is_terminal()),
        )
        .with(tracing_error::ErrorLayer::default())
        .try_init()?;

    trace!("{:?}", cli);

    // Call before reading the config
    if let Some(Command::Utils { command }) = &cli.command {
        return command.run();
    }

    let config = Config::read().wrap_err("couldn't read configuration")?;

    debug!("{:?}", config);

    match cli.command {
        Some(command) => match command {
            Command::Edit(edit) => note(&config, &edit.path),
            Command::Journal { date } => journal(&config, date.as_deref()),
            Command::Work { date } => work(&config, date.as_deref()),
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
            Command::List { path, max_depth } => list_path(&config, path, max_depth),
            Command::Query { search } => query(&search, &config),
            Command::Utils { .. } => {
                unreachable!("already matched");
            }
        },
        None => note(&config, "inbox"),
    }
}
