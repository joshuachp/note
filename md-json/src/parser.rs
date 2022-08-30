//! Markdown parser using [`pulldown_cmark`]
//!
//! ### Html
//!
//! The content of HTML tags must not be separated by empty lines. Otherwise
//! the parser could parse it as different HTML events.

use chrono::NaiveDate;
use html_parse::html::Html;
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
    pub title: String,
    pub description: Option<String>,
    pub tags: Vec<String>,
    pub date: NaiveDate,
    pub draft: bool,
    pub language: Option<String>,
    pub content: Vec<Event<'a>>,
}

#[derive(Deserialize, Debug)]
pub struct FrontMatter {
    title: String,
    description: Option<String>,
    tags: Vec<String>,
    date: String,
    draft: Option<bool>,
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
        description,
        tags,
        date,
        draft,
        language,
    } = metadata;

    let date = NaiveDate::parse_from_str(&date, "%Y-%m-%d").map_err(Error::Date)?;

    let options = Options::all();

    let mut html = Html::parse_fragment(content);

    let fragment = html.tree.first_node_id().unwrap();
    html.tree.remove(fragment).unwrap();
    let html_tag = html.tree.first_node_id().unwrap();
    html.tree.remove(html_tag).unwrap();

    println!("{}", serde_json::to_string(&html).unwrap());
    panic!();

    Ok(Markdown {
        title,
        description,
        tags,
        date,
        draft: draft.unwrap_or(false),
        language,
        content: Vec::new(),
    })
}

#[cfg(test)]
mod test {
    use std::fs;

    use super::{front_matter_parser, parse};

    #[test]
    fn should_parse_front_matter() {
        let markdown = include_str!("../assets/front_matter.md");

        let result = front_matter_parser(markdown);

        assert_eq!(result, Ok(("\n# Hello world\n", "title: \"some\"\n")));
    }

    #[test]
    fn should_parse_html() {
        let html = include_str!("../assets/html.md");

        parse(html).unwrap();
    }
}
