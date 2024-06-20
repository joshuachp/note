use std::{
    fs,
    path::{Path, PathBuf},
};

use color_eyre::eyre::{self, ensure, Context, OptionExt};
use dirs::cache_dir;
use log::{debug, error, trace};
use tantivy::{
    collector::TopDocs,
    directory::MmapDirectory,
    doc,
    query::QueryParser,
    schema::{Field, SchemaBuilder, STORED, TEXT},
    DocAddress, Document, Index, IndexWriter, Score, TantivyDocument,
};
use walkdir::WalkDir;

use crate::{config::Config, list::is_hidden};

struct MdFile {
    path: PathBuf,
    title: String,
    description: String,
    content: String,
}

struct SchemaFields {
    path: Field,
    title: Field,
    description: Field,
    content: Field,
}

impl SchemaFields {
    fn write(&self, writer: &IndexWriter, file: MdFile) -> eyre::Result<()> {
        let path = file.path.to_str().ok_or_eyre("invalid non utf-8 path")?;

        writer.add_document(doc! {
            self.path  => path,
            self.title => file.title,
            self.description  => file.description,
            self.content  => file.content,
        })?;

        Ok(())
    }
}

fn build_schema() -> (tantivy::schema::Schema, SchemaFields) {
    let mut schema_builder = SchemaBuilder::new();

    let title = schema_builder.add_text_field("title", TEXT | STORED);
    let path = schema_builder.add_text_field("path", TEXT | STORED);
    let description = schema_builder.add_text_field("description", TEXT);
    let content = schema_builder.add_text_field("content", TEXT);

    (
        schema_builder.build(),
        SchemaFields {
            title,
            path,
            description,
            content,
        },
    )
}

pub fn query(search: &str, config: &Config) -> eyre::Result<()> {
    let mut cache_dir = cache_dir().ok_or_eyre("missing cache dir")?;

    cache_dir.push("note");
    cache_dir.push("index");

    if cache_dir.exists() {
        std::fs::remove_dir_all(&cache_dir).wrap_err("failed to create cache directory")?;
    }
    std::fs::create_dir_all(&cache_dir).wrap_err("failed to create cache directory")?;

    let (schema, fields) = build_schema();

    let mmap_dir = MmapDirectory::open(&cache_dir)?;

    let index = Index::open_or_create(mmap_dir, schema.clone())?;

    let mut index_writer: IndexWriter = index.writer(100_000_000)?;

    let md_files = read_notes(&config.note_path)?;

    for md_file in md_files {
        fields.write(&index_writer, md_file)?;
    }

    index_writer.commit()?;

    let reader = index.reader()?;

    let searcher = reader.searcher();

    let query_parser = QueryParser::for_index(&index, vec![fields.title, fields.description]);

    // QueryParser may fail if the query is not in the right
    // format. For user facing applications, this can be a problem.
    // A ticket has been opened regarding this problem.
    let query = query_parser.parse_query(search)?;

    // Perform search.
    // `topdocs` contains the 10 most relevant doc ids, sorted by decreasing scores...
    let top_docs: Vec<(Score, DocAddress)> = searcher.search(&query, &TopDocs::with_limit(10))?;

    for (score, doc_address) in top_docs {
        // Retrieve the actual content of documents given its `doc_address`.
        let retrieved_doc = searcher.doc::<TantivyDocument>(doc_address)?;
        println!("{score}\t{}", retrieved_doc.to_json(&schema));
    }
    Ok(())
}

fn read_notes(path: &Path) -> eyre::Result<Vec<MdFile>> {
    debug!("reading {}", path.display());

    ensure!(path.exists(), "path doesn't exits");

    let iter = WalkDir::new(path)
        .into_iter()
        .filter_entry(|e| !is_hidden(e));

    let mut files = Vec::new();
    for entry in iter {
        let entry = entry?;

        debug!("checking {}", entry.path().display());

        // Filter for markdown files
        if !(entry.path().extension().is_some_and(|ext| ext == "md") && entry.path().is_file()) {
            debug!("skipping {}", entry.path().display());

            continue;
        }

        let path = entry.path();

        trace!("{}", path.display());

        let content = fs::read_to_string(path)?;
        let markdown = match md_parser::parser::parse(&content) {
            Ok(m) => m,
            Err(err) => {
                error!(
                    "culdn't parse the file {}: {:#}",
                    entry.path().display(),
                    err
                );

                continue;
            }
        };

        trace!("file: `{}` content: `{}`", path.display(), content);

        let content = markdown.content_into_string();

        files.push(MdFile {
            path: path.to_owned(),
            title: markdown.title,
            description: markdown.description,
            content,
        })
    }

    Ok(files)
}
