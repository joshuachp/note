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
use pulldown_cmark::{CowStr, Event, Options, Parser, Tag};
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
    pub content: Vec<MdEvent<'a>>,
}

#[derive(Serialize, Debug)]
pub enum MdEvent<'a> {
    #[cfg_attr(feature = "serde", serde(borrow))]
    Start(Tag<'a>),
    #[cfg_attr(feature = "serde", serde(borrow))]
    End(Tag<'a>),
    #[cfg_attr(feature = "serde", serde(borrow))]
    Text(CowStr<'a>),
    #[cfg_attr(feature = "serde", serde(borrow))]
    Code(CowStr<'a>),
    #[cfg_attr(feature = "serde", serde(borrow))]
    Html(Html),
    #[cfg_attr(feature = "serde", serde(borrow))]
    FootnoteReference(CowStr<'a>),
    SoftBreak,
    HardBreak,
    Rule,
    TaskListMarker(bool),
}

impl<'a> From<Event<'a>> for MdEvent<'a> {
    fn from(value: Event<'a>) -> Self {
        match value {
            Event::Start(value) => MdEvent::Start(value),
            Event::End(value) => MdEvent::End(value),
            Event::Text(value) => MdEvent::Text(value),
            Event::Code(value) => MdEvent::Code(value),
            Event::FootnoteReference(value) => MdEvent::FootnoteReference(value),
            Event::SoftBreak => MdEvent::SoftBreak,
            Event::HardBreak => MdEvent::HardBreak,
            Event::Rule => MdEvent::Rule,
            Event::TaskListMarker(value) => MdEvent::TaskListMarker(value),
            Event::Html(fragment) => {
                let html = Html::parse_fragment(&fragment);

                MdEvent::Html(html)
            }
        }
    }
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
    let parser = Parser::new_ext(content, options).map(MdEvent::from);

    Ok(Markdown {
        title,
        description,
        tags,
        date,
        draft: draft.unwrap_or(false),
        language,
        content: parser.collect(),
    })
}

#[cfg(test)]
mod test {
    use super::{front_matter_parser, parse};

    #[test]
    fn should_parse_front_matter() {
        let markdown = r#"---
title: "some"
---

# Hello world"#;

        let result = front_matter_parser(markdown);

        assert_eq!(result, Ok(("\n# Hello world", "title: \"some\"\n")));
    }

    #[test]
    fn should_parse_html() {
        let html_test = include_str!("../assets/test_parse_html.md");

        let result = parse(html_test);

        dbg!(result.unwrap());

        panic!()
    }
}
