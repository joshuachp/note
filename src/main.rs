mod cli;
mod config;

use clap::Parser;
use config::Config;
use log::trace;

use crate::cli::Cli;

fn main() {
    env_logger::init();

    let cli = Cli::parse();

    trace!("{:?}", cli);

    let config = Config::read();

    trace!("{:?}", config);
}
