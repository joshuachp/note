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
    description: String,
    language: Option<String>,
    content: Vec<Event<'a>>,
}

#[derive(Deserialize, Debug)]
pub struct FrontMatter {
    title: String,
    tags: Vec<String>,
    description: String,
    language: Option<String>,
}

fn front_matter_parser(markdown: &str) -> IResult<&str, &str> {
    delimited(
        pair(tag("---"), line_ending),
        take_until("---"),
        pair(tag("---"), alt((line_ending, eof))),
    )(markdown)
}

pub fn front_matter(markdown: &str) -> Result<(FrontMatter, &str), Error> {
    match front_matter_parser(markdown) {
        Ok((rest, yaml)) => {
            let front_matter: FrontMatter =
                serde_yaml::from_str(yaml).map_err(Error::FrontMatter)?;
            Ok((front_matter, rest))
        }
        Err(_) => Err(Error::MissingFrontMatter),
    }
}

pub fn parse(markdown: &str) -> Result<Markdown, Error> {
    let (metadata, content) = front_matter(markdown)?;

    let FrontMatter {
        title,
        tags,
        description,
        language,
    } = metadata;

    let options = Options::all();
    let parser = Parser::new_ext(content, options);

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
