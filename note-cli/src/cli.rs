//! CLI interface and completion

use std::io;

use clap::{ArgEnum, Args, IntoApp, Parser, Subcommand, ValueHint};
use clap_complete::generate;

/// Note taking utility
#[derive(Debug, Parser)]
pub struct Cli {
    /// General sub commands
    #[clap(subcommand)]
    pub command: Option<Command>,
}

#[derive(Debug, Args)]
pub struct Edit {
    /// Title and name of the file
    #[clap(value_hint(ValueHint::FilePath))]
    pub path: String,
}

/// Possible sub commands
#[derive(Debug, Subcommand)]
pub enum Command {
    /// Edits a note
    #[clap(visible_alias("e"))]
    Edit(Edit),
    /// Opens the daily journal
    #[clap(visible_alias("j"))]
    Journal {
        /// Journal entry to edit, in the Y-m-d format
        date: Option<String>,
    },
    /// Opens the todo file
    #[clap(visible_alias("t"))]
    Todo,
    /// Search the content of the notes
    #[clap(visible_alias("s"))]
    Search {
        /// Content
        content: Option<String>,
    },
    /// Search the name of the files
    #[clap(visible_alias("f"))]
    Find {
        /// Filename
        filename: Option<String>,
    },
    /// Sync the notes using the configured sync command
    Sync,
    /// Prints the shell completion
    Completion {
        #[clap(arg_enum)]
        shell: Shell,
    },
    /// Compiles notes to JSON
    Compile {
        #[clap(value_hint(ValueHint::AnyPath))]
        path: String,
    },
}

#[derive(Debug, Clone, Copy, ArgEnum)]
pub enum Shell {
    Bash,
    Zsh,
    Fish,
}

pub fn generate_completion(shell: Shell) {
    let shell = match shell {
        Shell::Bash => clap_complete::Shell::Bash,
        Shell::Zsh => clap_complete::Shell::Zsh,
        Shell::Fish => clap_complete::Shell::Fish,
    };

    generate(shell, &mut Cli::command(), "note", &mut io::stdout())
}
