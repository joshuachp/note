use clap::Parser;
use log::trace;

use crate::cli::Cli;

mod cli;

fn main() {
    env_logger::init();

    let cli = Cli::parse();

    trace!("{:?}", cli)
}
