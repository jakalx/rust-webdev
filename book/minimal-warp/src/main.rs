use std::collections::HashMap;
use std::io::ErrorKind;
use std::str::FromStr;

use warp::Filter;

#[derive(PartialEq, PartialOrd, Debug, Clone)]
struct Question {
    id: QuestionId,
    title: Title,
    content: Content,
    tags: Option<Vec<Tag>>,
}

#[derive(PartialEq, PartialOrd, Debug, Clone)]
struct QuestionId(String);
#[derive(PartialEq, PartialOrd, Debug, Clone)]
struct Title(String);
#[derive(PartialEq, PartialOrd, Debug, Clone)]
struct Content(String);
#[derive(PartialEq, PartialOrd, Debug, Clone)]
struct Tag(String);

impl Question {
    fn new(id: QuestionId, title: Title, content: Content, tags: Option<Vec<Tag>>) -> Self {
        Question {
            id,
            title,
            content,
            tags,
        }
    }
}

impl std::fmt::Display for Question {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{}, title: {}, content: {}, tags: {:?}",
            self.id, self.title, self.content, self.tags
        )
    }
}

impl std::fmt::Display for QuestionId {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for QuestionId {
    type Err = std::io::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.is_empty() {
            true => Err(Self::Err::new(ErrorKind::InvalidInput, "empty id")),
            false => Ok(Self(s.to_string())),
        }
    }
}

impl std::fmt::Display for Title {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::fmt::Display for Content {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::fmt::Display for Tag {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "#{}", self.0)
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let question = Question::new(
        QuestionId::from_str("q1")?,
        Title("title".into()),
        Content("foo".into()),
        None,
    );

    let resp = reqwest::get("https://httpbin.org/ip")
        .await?
        .json::<HashMap<String, String>>()
        .await?;
    println!("{:?}", resp);

    let hello = warp::path("hello")
        .and(warp::path::param())
        .map(move |name: String| format!("{} asked {}\n", name, question));

    warp::serve(hello).run(([127, 0, 0, 1], 1337)).await;

    Ok(())
}
