use std::io::ErrorKind;
use std::str::FromStr;

use serde::Serialize;

use warp::http::StatusCode;
use warp::reject::Reject;
use warp::Filter;
use warp::Rejection;
use warp::Reply;

#[derive(PartialEq, PartialOrd, Debug, Clone, Serialize)]
struct Question {
    id: QuestionId,
    title: Title,
    content: Content,
    tags: Option<Vec<Tag>>,
}

#[derive(PartialEq, PartialOrd, Debug, Clone, Serialize)]
struct QuestionId(String);
#[derive(PartialEq, PartialOrd, Debug, Clone, Serialize)]
struct Title(String);
#[derive(PartialEq, PartialOrd, Debug, Clone, Serialize)]
struct Content(String);
#[derive(PartialEq, PartialOrd, Debug, Clone, Serialize)]
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

#[derive(Debug)]
struct InvalidId;
impl Reject for InvalidId {}

async fn get_questions() -> Result<impl warp::Reply, warp::Rejection> {
    let question = Question::new(
        QuestionId::from_str("1").expect("valid id"),
        Title("The Question".into()),
        Content("The answer to life, the universe and everything?".into()),
        Some(vec![Tag("h2g2".into())]),
    );

    match question.id.0.parse::<u32>() {
        Err(_) => Err(warp::reject::custom(InvalidId)),
        Ok(_) => Ok(warp::reply::json(&question)),
    }
}

async fn return_error(r: Rejection) -> Result<impl Reply, Rejection> {
    if let Some(InvalidId) = r.find() {
        Ok(warp::reply::with_status(
            "No valid ID presented",
            StatusCode::UNPROCESSABLE_ENTITY,
        ))
    } else {
        Ok(warp::reply::with_status(
            "Route not found",
            StatusCode::NOT_FOUND,
        ))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let get_items = warp::get()
        .and(warp::path("questions"))
        .and(warp::path::end())
        .and_then(get_questions)
        .recover(return_error);

    let routes = get_items;

    warp::serve(routes).run(([127, 0, 0, 1], 1337)).await;

    Ok(())
}
