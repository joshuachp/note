mod cli;
mod config;

use clap::Parser;
use config::Config;
use log::{error, trace};

use crate::cli::Cli;

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
}
