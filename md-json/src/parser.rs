use nom::{
    branch::alt,
    bytes::complete::{tag, take_until},
    character::streaming::line_ending,
    combinator::eof,
    sequence::{delimited, pair},
    IResult,
};
use pulldown_cmark::{Event, Options, Parser};
use serde::{Deserialize, Serialize};

use crate::error::Error;

#[derive(Serialize, Debug)]
pub struct Markdown<'a> {
    title: String,
    tags: Vec<String>,
    description: Option<String>,
    language: Option<String>,
    content: Vec<Event<'a>>,
}

#[derive(Deserialize, Debug)]
pub struct FrontMatter {
    title: String,
    tags: Vec<String>,
    description: Option<String>,
    language: Option<String>,
}

fn front_matter_parser(markdown: &str) -> IResult<&str, &str> {
    delimited(
        pair(tag("---"), line_ending),
        take_until("---"),
        pair(tag("---"), alt((line_ending, eof))),
    )(markdown)
}

pub fn front_matter<'a>(markdown: &'a str) -> Result<Option<FrontMatter>, Error> {
    match front_matter_parser(markdown) {
        Ok((_, yaml)) => {
            let front_matter: FrontMatter =
                serde_yaml::from_str(yaml).map_err(|err| Error::FrontMatter(err))?;

            Ok(Some(front_matter))
        }
        Err(_) => Ok(None),
    }
}

pub fn parse(markdown: &str) -> Result<Markdown, Error> {
    let options = Options::all();
    let parser = Parser::new_ext(markdown, options);

    // TODO: handle None case
    let FrontMatter {
        title,
        tags,
        description,
        language,
    } = front_matter(markdown)?.unwrap_or_else(|| FrontMatter {
        title: String::from(""),
        tags: Vec::new(),
        description: None,
        language: None,
    });

    Ok(Markdown {
        title,
        tags,
        description,
        language,
        content: parser.collect(),
    })
}

#[cfg(test)]
mod test {
    use super::front_matter_parser;

    #[test]
    fn should_parse_front_matter() {
        let markdown = r#"---
title: "some"
---

# Hello world"#;

        let result = front_matter_parser(markdown);

        assert_eq!(result, Ok(("\n# Hello world", "title: \"some\"\n")));
    }
}
