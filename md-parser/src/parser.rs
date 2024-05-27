use std::{collections::HashSet, str::FromStr};

use chrono::NaiveDate;
use pulldown_cmark::{Event, Options};
use winnow::{
    ascii::line_ending,
    combinator::{alt, delimited, eof},
    token::take_until,
    PResult, Parser,
};
use yaml_rust2::Yaml;

use crate::error::Error;

#[derive(thiserror::Error, Debug)]
pub enum FrontMatterError {
    #[error("failed to parse yaml")]
    Yaml(#[from] yaml_rust2::ScanError),
    #[error("yaml contains multiple documents")]
    MultipleDoc,
    #[error("empty front matter")]
    Empty,
    #[error("expected and map")]
    Map,
    #[error("missing {0}")]
    FieldMissing(&'static str),
    #[error("expected {name} to be a {expected}")]
    FieldType {
        name: &'static str,
        expected: &'static str,
    },
    #[error("invalid language")]
    Language(#[from] LanguageError),
    #[error("invalid date for {name}")]
    Date {
        name: &'static str,
        #[source]
        backtrace: chrono::ParseError,
    },
}

impl FrontMatterError {
    const fn missing(missing: &'static str) -> Self {
        Self::FieldMissing(missing)
    }

    const fn field_type(name: &'static str, expected: &'static str) -> Self {
        Self::FieldType { name, expected }
    }
}

#[derive(Debug)]
pub struct Markdown<'a> {
    pub title: String,
    pub description: String,
    pub tags: HashSet<String>,
    pub created: NaiveDate,
    pub updated: Option<NaiveDate>,
    pub released: bool,
    pub language: Option<Language>,
    pub content: Vec<Event<'a>>,
}

#[derive(Debug, Clone)]
pub struct FrontMatter {
    title: String,
    description: String,
    tags: HashSet<String>,
    created: NaiveDate,
    updated: Option<NaiveDate>,
    released: bool,
    language: Option<Language>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Language {
    Eng,
    It,
}

impl FromStr for Language {
    type Err = LanguageError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let lang = match s {
            "en" => Language::Eng,
            "it" => Language::It,
            invalid => return Err(LanguageError(invalid.to_string())),
        };

        Ok(lang)
    }
}

#[derive(Debug, thiserror::Error)]
#[error("invalid language {0}")]
pub struct LanguageError(String);

fn front_matter(markdown: &mut &str) -> PResult<FrontMatter> {
    delimited(
        ("---", line_ending),
        take_until(1.., "---").try_map(parse_front_matter),
        ("---", alt((line_ending, eof))),
    )
    .parse_next(markdown)
}

fn parse_front_matter(source: &str) -> Result<FrontMatter, FrontMatterError> {
    let mut scan = yaml_rust2::YamlLoader::load_from_str(source)?;

    dbg!(&scan);

    if scan.is_empty() {
        return Err(FrontMatterError::Empty);
    }

    if scan.len() > 1 && scan[1] != Yaml::Null {
        return Err(FrontMatterError::MultipleDoc);
    }

    let doc = scan.swap_remove(0);

    let mut map = doc.into_hash().ok_or(FrontMatterError::Map)?;

    let title = map
        .remove(&Yaml::from_str("title"))
        .ok_or(FrontMatterError::missing("title"))?
        .into_string()
        .ok_or(FrontMatterError::field_type("title", "string"))?;

    let description = map
        .remove(&Yaml::from_str("description"))
        .ok_or(FrontMatterError::missing("description"))?
        .into_string()
        .ok_or(FrontMatterError::field_type("description", "string"))?;

    let tags = match map.remove(&Yaml::from_str("tags")) {
        Some(tags) => {
            let tags_array = tags
                .into_vec()
                .ok_or(FrontMatterError::field_type("tags", "array"))?;

            tags_array
                .into_iter()
                .map(|v| {
                    v.into_string()
                        .ok_or(FrontMatterError::field_type("tags", "string"))
                })
                .collect::<Result<_, _>>()?
        }
        None => HashSet::new(),
    };

    let created = map
        .get(&Yaml::from_str("created"))
        .ok_or(FrontMatterError::missing("created"))?
        .as_str()
        .ok_or(FrontMatterError::field_type("created", "string"))
        .and_then(|date| {
            NaiveDate::parse_from_str(date, "%Y-%m-%d").map_err(|err| FrontMatterError::Date {
                name: "created",
                backtrace: err,
            })
        })?;

    let updated = map
        .remove(&Yaml::from_str("updated"))
        .map(|updated| {
            updated
                .as_str()
                .ok_or(FrontMatterError::field_type("created", "string"))
                .and_then(|date| {
                    NaiveDate::parse_from_str(date, "%Y-%m-%d").map_err(|err| {
                        FrontMatterError::Date {
                            name: "updated",
                            backtrace: err,
                        }
                    })
                })
        })
        .transpose()?;

    let released = map
        .get(&Yaml::from_str("released"))
        .map(|released| {
            released
                .as_bool()
                .ok_or(FrontMatterError::field_type("released", "bool"))
        })
        .transpose()?
        .unwrap_or_default();

    let language = map
        .get(&Yaml::from_str("language"))
        .map(|language| {
            language
                .as_str()
                .ok_or(FrontMatterError::field_type("language", "string"))
                .and_then(|lang| Language::from_str(lang).map_err(FrontMatterError::Language))
        })
        .transpose()?;

    Ok(FrontMatter {
        title,
        description,
        tags,
        created,
        updated,
        released,
        language,
    })
}

pub fn parse(mut markdown: &str) -> Result<Markdown, Error> {
    let metadata = front_matter.parse_next(&mut markdown).map_err(|err| {
        err.into_inner()
            .map(Error::FrontMatter)
            .unwrap_or(Error::MissingFrontmatter)
    })?;

    let FrontMatter {
        title,
        description,
        tags,
        created,
        updated,
        released,
        language,
    } = metadata;

    let options = Options::all();
    let parser = pulldown_cmark::Parser::new_ext(markdown, options);

    Ok(Markdown {
        title,
        description,
        tags,
        created,
        updated,
        released,
        language,
        content: parser.collect(),
    })
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn should_parse_front_matter() {
        let markdown = r#"---
title: "foo"
# comment
description: "bar"
tags: ["tag1", "tag1", "tag2"]
created: "1970-01-01"
updated: "1970-01-01"
released: false
language: en
---

# Hello world"#;

        let result = parse_front_matter(markdown).unwrap();

        assert_eq!(result.title, "foo");
        assert_eq!(result.description, "bar");
        assert_eq!(
            result.tags,
            HashSet::from_iter(["tag1".to_string(), "tag2".to_string()])
        );
        assert_eq!(result.created, NaiveDate::from_ymd_opt(1970, 1, 1).unwrap());
        assert_eq!(result.updated, NaiveDate::from_ymd_opt(1970, 1, 1));
        assert!(!result.released);
        assert_eq!(result.language, Some(Language::Eng));
    }
}
