//! CLI interface and completion

use std::path::PathBuf;

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
    #[command(visible_alias("e"))]
    Edit(Edit),
    /// Opens the daily journal
    #[command(visible_alias("j"))]
    Journal {
        /// Journal entry to edit, in the Y-m-d format
        date: Option<String>,
    },
    /// Opens the todo file
    #[command(visible_alias("t"))]
    Todo,
    /// Search the content of the notes
    #[command(visible_alias("s"))]
    Search {
        /// Content
        content: Option<String>,
    },
    /// Search the name of the files
    #[command(visible_alias("f"))]
    Find {
        /// Filename
        filename: Option<String>,
    },
    /// Full text search of all the notes.
    #[clap(visible_alias("q"))]
    Query {
        // The search query.
        search: String,
    },
    /// Sync the notes using the configured sync command
    Sync,
    /// List the notes in $NOTE_PATH or the current directory.
    #[command(visible_alias("ls"))]
    List {
        /// Path to list.
        #[arg(value_hint(ValueHint::DirPath))]
        path: Option<PathBuf>,

        #[arg(short = 'd', long, default_value = "1")]
        max_depth: usize,
    },
    /// Utility functions like shell completions
    Utils {
        #[command(subcommand)]
        command: Utils,
    },
}

#[derive(Debug, Subcommand)]
pub enum Utils {
    /// Generates shell completions for the given shell
    Completion {
        /// Shell to generate the completion for
        shell: Shell,
    },
    /// Generates the Man pages for the program
    Manpages {
        /// Directory to generate the Man pages to.
        out_dir: PathBuf,
    },
}

impl Utils {
    pub(crate) fn run(&self) -> eyre::Result<()> {
        match self {
            Utils::Completion { shell } => shell.generate(),
            Utils::Manpages { out_dir } => {
                clap_mangen::generate_to(Cli::command(), out_dir)?;

                Ok(())
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum Shell {
    Bash,
    Zsh,
    Fish,
}

impl Shell {
    fn generate(&self) -> Result<()> {
        let completion_shell = match self {
            Shell::Bash => clap_complete::Shell::Bash,
            Shell::Zsh => clap_complete::Shell::Zsh,
            Shell::Fish => clap_complete::Shell::Fish,
        };

        let mut buff: Vec<u8> = Vec::new();

        generate(completion_shell, &mut Cli::command(), "note", &mut buff);

        let mut completion = String::from_utf8(buff)?;

        if *self == Shell::Zsh {
            let pattern = Regex::new(r#"(?m)^'(.*:_files)'"#)?;
            completion = pattern
                .replace_all(&completion, r#""$1 -W $$NOTE_PATH -g '*.md'""#)
                .to_string();
        }

        println!("{completion}");

        if *self == Shell::Fish {
            println!(include_str!("../shell/__note_list_completion.fish"));
            println!(
                r#"complete -c note -n "__fish_seen_subcommand_from e edit" -k -f -a '(__note_list_completion)'"#
            )
        }

        Ok(())
    }
}
