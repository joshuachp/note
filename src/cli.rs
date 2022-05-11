//! CLI interface and completion

use clap::{ArgEnum, Parser, Subcommand};

/// Note taking utility
#[derive(Debug, Parser)]
pub struct Cli {
    /// General sub commands
    #[clap(subcommand)]
    pub command: Command,
}

/// Possible sub commands
#[derive(Debug, Subcommand)]
pub enum Command {
    /// Opens the daily journal
    Journal,
    /// Opens the todo file
    Todo,
    /// Search the content of the notes
    Search,
    /// Search the name of the files
    Find,
    /// Sync the notes using the configured sync command
    Sync,
    /// Prints the shell completion
    Completion {
        #[clap(arg_enum)]
        shell: Shell,
    },
}

#[derive(Debug, Clone, Copy, ArgEnum)]
pub enum Shell {
    Bash,
    Zsh,
    Fish,
}
