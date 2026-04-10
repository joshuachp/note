use std::io::{BufRead, BufReader, BufWriter, Read, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::str::FromStr;
use std::{fs, io};

use askama::Template;
use chrono::{Local, NaiveDate};
use eyre::{self, Context, ensure};

use sha2::Digest;
use tracing::{debug, error, info, instrument, trace, warn};

use crate::config::Config;

/// Edit a note
#[instrument(skip(config))]
pub fn note(config: &Config, path: &str) -> eyre::Result<()> {
    let note_path = NoteArgs::parse(&config.note_path, path)?;

    let note = Note::now(note_path.title);

    note.edit(config, &note_path.path)
}

/// Edit a journal entry
#[instrument(skip(config))]
pub fn journal(config: &Config, date: Option<&str>) -> eyre::Result<()> {
    let entry = JournalArgs::entry("journal", date)?;

    let mut note = Note::now(format!("Journal {}", entry.date));

    note.description = Some(format!("Daily notes for the {}", entry.date));
    note.tags.push("journal".to_string());
    note.lang = Some("en".to_string());

    note.edit(config, &entry.path)
}

/// Edit a work journal entry
#[instrument(skip(config))]
pub fn work(config: &Config, date: Option<&str>) -> eyre::Result<()> {
    let entry = JournalArgs::entry("work", date)?;

    let mut note = Note::now(format!("Work {}", entry.date));

    note.description = Some(format!("Work daily notes for the {}", entry.date));
    note.tags.extend(["journal", "work"].map(str::to_string));
    note.lang = Some("en".to_string());

    note.edit(config, &entry.path)
}

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

    fn update(&mut self, buf: &[u8]) -> io::Result<()> {
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

    #[instrument(skip(self))]
    fn create_note(&self, file: &Path) -> eyre::Result<()> {
        let file = fs::File::options()
            .write(true)
            .create_new(true)
            .open(file)?;

        let mut file = BufWriter::new(file);

        self.write_into(&mut file)?;

        file.flush()?;

        info!("note created");

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

    #[instrument(skip(self, config))]
    fn edit(&self, config: &Config, note_path: &Path) -> eyre::Result<()> {
        let abs_path = config.note_path.join(note_path);

        // TODO: if parent doesn't exist create a temp file and then move it if file exists after edit
        if let Some(parent) = abs_path.parent() {
            trace!("Parent {:?}", parent);

            if !parent.is_dir() {
                warn!("Creating parent directory: {parent:?}");

                fs::create_dir_all(parent).context("failed to create parent directories")?;
            }
        }

        // Ensure path, if it exists, it has to be a file.
        let is_new = match fs::metadata(&abs_path) {
            Ok(metadata) => {
                ensure!(metadata.is_file(), "not a file");

                false
            }
            Err(err) if err.kind() == io::ErrorKind::NotFound => {
                debug!(file = %abs_path.display(), "file does not exists");

                self.create_note(&abs_path)?;

                true
            }
            Err(err) => {
                return Err(err)
                    .wrap_err_with(|| format!("reading file metadata: {}", abs_path.display()));
            }
        };

        let status = Command::new(&config.editor)
            .args([&note_path])
            .current_dir(&config.note_path)
            .spawn()
            .context("failed to spawn editor")?
            .wait()
            .context("failed to wait to editor")?;

        trace!(%status, "editior exited");

        ensure!(
            status.success(),
            "editor returned with status code {status}"
        );

        if is_new && self.is_template(&abs_path)? {
            debug!("file was not edited, removing");

            fs::remove_file(&abs_path)?;
        }

        Ok(())
    }
}

#[derive(Debug)]
struct NoteArgs {
    title: String,
    path: PathBuf,
}

impl NoteArgs {
    #[instrument(ret)]
    fn parse(base_path: &Path, path: &str) -> eyre::Result<Self> {
        let path = path.trim();

        // No need to check whitespace since we trimmed
        if path.is_empty() {
            return Err(eyre::eyre!("note name cannot be empty"));
        }

        let title = Self::note_title(path);
        let path = Self::file_path(base_path, path);

        Ok(Self { title, path })
    }

    fn note_title(path: &str) -> String {
        let title = path.replace('_', " ");

        let (first, rest) = title.split_at(1);

        let mut title = first.to_uppercase();

        title += rest;

        title
    }

    fn file_path(base_path: &Path, path: &str) -> PathBuf {
        let path_str: String = path
            .chars()
            .map(|chr| {
                if chr.is_whitespace() {
                    '_'
                } else {
                    chr.to_ascii_lowercase()
                }
            })
            .collect();

        let mut file_path = PathBuf::from(base_path);
        file_path.push(&path_str);
        file_path.set_extension("md");

        file_path
    }
}

#[derive(Debug)]
struct JournalArgs {
    date: NaiveDate,
    path: PathBuf,
}

impl JournalArgs {
    fn entry(base: impl AsRef<Path>, date: Option<&str>) -> eyre::Result<Self> {
        let date = match date {
            Some(date) => NaiveDate::from_str(date)
                .wrap_err_with(|| format!("failed to parse date {date}"))?,
            None => Local::now().date_naive(),
        };

        let mut path = PathBuf::from(base.as_ref());
        path.push(date.to_string());
        path.set_extension("md");

        Ok(Self { date, path })
    }
}
