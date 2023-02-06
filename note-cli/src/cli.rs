//! CLI interface and completion

use clap::{Args, CommandFactory, Parser, Subcommand, ValueEnum, ValueHint};
use clap_complete::generate;
use color_eyre::Result;
use regex::Regex;

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
        #[clap(value_enum)]
        shell: Shell,
    },
    /// Compiles notes to JSON
    Compile {
        #[clap(value_hint(ValueHint::AnyPath))]
        path: String,

        /// Include drafts in the compiled JSON
        #[clap(long = "draft")]
        drafts: bool,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum Shell {
    Bash,
    Zsh,
    Fish,
}

pub fn generate_completion(shell: Shell) -> Result<()> {
    let completion_shell = match shell {
        Shell::Bash => clap_complete::Shell::Bash,
        Shell::Zsh => clap_complete::Shell::Zsh,
        Shell::Fish => clap_complete::Shell::Fish,
    };

    let mut buff: Vec<u8> = Vec::new();

    generate(completion_shell, &mut Cli::command(), "note", &mut buff);

    let mut completion = String::from_utf8(buff)?;

    if shell == Shell::Zsh {
        let pattern = Regex::new(r#"(?m)^'(.*:_files)'"#)?;
        completion = pattern
            .replace_all(&completion, r#""$1 -W $$NOTE_PATH -g '*.md'""#)
            .to_string();
    }

    print!("{completion}");

    Ok(())
}
