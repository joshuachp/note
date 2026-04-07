use std::io::{BufRead, BufReader, BufWriter, Read, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::str::FromStr;
use std::{fs, io};

use askama::Template;
use chrono::{Local, NaiveDate};
use color_eyre::Result;
use eyre::{Context, ContextCompat, ensure};

use sha2::Digest;
use tracing::{debug, error, instrument, trace, warn};

use crate::config::Config;

#[derive(Debug)]
struct State {
    bytes: u64,
    hash: sha2::Sha256,
}

impl State {
    fn new() -> Self {
        Self {
            bytes: 0,
            hash: sha2::Sha256::new(),
        }
    }

    fn update(&mut self, buf: &[u8]) -> Result<(), io::Error> {
        self.hash.update(buf);
        self.bytes += u64::try_from(buf.len()).map_err(|error| {
            error!(%error,"couldn't convert written bytes");

            io::Error::from(io::ErrorKind::FileTooLarge)
        })?;
        Ok(())
    }
}

#[derive(Debug)]
struct FileHash<F> {
    state: State,
    inner: F,
}

impl<F> FileHash<F> {
    fn new(inner: F) -> Self {
        Self {
            state: State::new(),
            inner,
        }
    }
}

impl FileHash<io::Sink> {
    fn with_sink() -> Self {
        Self::new(io::sink())
    }
}

impl<R> Read for FileHash<R>
where
    R: Read,
{
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let read = self.inner.read(buf)?;

        self.state.update(&buf[..read])?;

        Ok(read)
    }
}

impl<R> io::BufRead for FileHash<R>
where
    R: io::BufRead,
{
    fn fill_buf(&mut self) -> io::Result<&[u8]> {
        let buf = self.inner.fill_buf()?;

        self.state.update(buf)?;

        Ok(buf)
    }

    fn consume(&mut self, amount: usize) {
        self.inner.consume(amount);
    }
}

impl<W> Write for FileHash<W>
where
    W: Write,
{
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let written = self.inner.write(buf)?;

        self.state.update(&buf[..written])?;

        Ok(written)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.inner.flush()
    }
}

#[derive(Debug, Template)]
#[template(path = "note.md")]
struct Note {
    title: String,
    description: Option<String>,
    created_at: String,
    updated_at: Option<String>,
    lang: Option<String>,
    tags: Vec<String>,
}

impl Note {
    fn now(title: String) -> Self {
        Self {
            title,
            description: None,
            created_at: Local::now().format("%Y-%m-%d").to_string(),
            updated_at: None,
            lang: None,
            tags: Vec::new(),
        }
    }

    #[instrument(skip(config))]
    fn with_path(config: &Config, path: &str) -> eyre::Result<Self> {
        let path = path.trim();
        let path = get_path(&config.note_path, path);

        let title = path
            .file_prefix()
            .wrap_err("missing file name")?
            .to_str()
            .wrap_err("non UTF-8 file name")?;

        let title = title.replace('_', " ");

        let (first, rest) = title.split_at(1);

        let mut title = first.to_uppercase();

        title += rest;

        Ok(Self::now(title))
    }

    #[instrument(skip(self))]
    fn create_note(&self, file: &Path) -> eyre::Result<()> {
        let file = fs::File::options()
            .write(true)
            .create_new(true)
            .open(file)?;

        let mut file = BufWriter::new(file);

        self.write_into(&mut file)?;

        Ok(())
    }

    // TODO: modification time
    #[instrument(skip(self))]
    fn is_template(&self, file: &Path) -> eyre::Result<bool> {
        let mut sink = FileHash::with_sink();

        self.write_into(&mut sink)?;

        let file = fs::File::open(file)?;

        let meta = file.metadata()?;

        if meta.len() != sink.state.bytes {
            return Ok(false);
        }

        let mut file = FileHash::new(BufReader::new(file));

        while let buf = file.fill_buf()?
            && !buf.is_empty()
        {
            let len = buf.len();
            file.consume(len);
        }

        Ok(file.state.hash.finalize() == sink.state.hash.finalize())
    }

    #[instrument]
    fn edit(&self, config: &Config, path: &str) -> eyre::Result<()> {
        let path = path.trim();

        trace!(path);

        let file_path = get_path(&config.note_path, path);

        // TODO: if parent doesn't exist create a temp file and then move it if file exists after edit
        if let Some(parent) = file_path.parent() {
            trace!("Parent {:?}", parent);

            if !parent.is_dir() {
                warn!("Creating parent directory: {parent:?}");

                fs::create_dir_all(parent).context("failed to create parent directories")?;
            }
        }

        // Ensure path, if it exists, it has to be a file.
        let _is_new = match fs::metadata(&file_path) {
            Ok(metadata) => {
                ensure!(metadata.is_file(), "not a file");

                false
            }
            Err(err) if err.kind() == io::ErrorKind::NotFound => {
                debug!(file = %file_path.display(), "file does not exists");

                self.create_note(&file_path)?;

                true
            }
            Err(err) => {
                return Err(err).wrap_err_with(|| format!("reading file metadata: {file_path:?}"));
            }
        };

        let mut command = Command::new(&config.editor);
        command.args([&file_path]);

        if config.change_dir {
            command.current_dir(&config.note_path);
        }

        let status = command
            .spawn()
            .context("failed to spawn editor")?
            .wait()
            .context("failed to wait to editor")?;

        trace!(%status, "editior exited");

        ensure!(
            status.success(),
            "editor returned with status code {status}"
        );

        if self.is_template(&file_path)? {
            debug!("file was not edited, removing");

            fs::remove_file(&file_path)?;
        }

        Ok(())
    }
}

#[instrument]
fn get_path(base_path: &Path, path: &str) -> PathBuf {
    let mut file_path = PathBuf::from(base_path);

    let path_string: String = path
        .chars()
        .map(|chr| {
            if chr.is_whitespace() {
                '_'
            } else {
                chr.to_ascii_lowercase()
            }
        })
        .collect();

    trace!("{}", path_string);

    file_path.push(path_string);
    file_path.set_extension("md");

    trace!(file_path = %file_path.display());

    file_path
}

#[instrument(skip(config))]
pub fn note(config: &Config, path: &str) -> Result<()> {
    let note = Note::with_path(config, path)?;

    note.edit(config, path)
}

#[instrument(skip(config))]
pub fn journal(config: &Config, base: &str, date: Option<&str>) -> Result<()> {
    let date = match date {
        Some(date) => {
            NaiveDate::from_str(date).wrap_err_with(|| format!("failed to parse date {date}"))?
        }
        None => Local::now().date_naive(),
    };

    let mut entry = PathBuf::from(base);
    entry.push(date.to_string());
    entry.set_extension("md");

    let path = entry.to_string_lossy();
    let note = Note::with_path(config, &path)?;

    note.edit(config, &path)
}
