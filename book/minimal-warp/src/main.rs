use std::{collections::HashMap, fmt::Display};

use serde::{Deserialize, Serialize};

use warp::{
    cors::CorsForbidden, http::Method, http::StatusCode, reject::Reject, Filter, Rejection, Reply,
};

#[derive(PartialEq, PartialOrd, Debug, Clone, Serialize, Deserialize)]
struct Question {
    id: QuestionId,
    title: Title,
    content: Content,
    tags: Option<Vec<Tag>>,
}

#[derive(PartialEq, PartialOrd, Eq, Hash, Debug, Clone, Serialize, Deserialize)]
struct QuestionId(String);
#[derive(PartialEq, PartialOrd, Debug, Clone, Serialize, Deserialize)]
struct Title(String);
#[derive(PartialEq, PartialOrd, Debug, Clone, Serialize, Deserialize)]
struct Content(String);
#[derive(PartialEq, PartialOrd, Debug, Clone, Serialize, Deserialize)]
struct Tag(String);

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

#[derive(Clone)]
struct Store {
    questions: HashMap<QuestionId, Question>,
}

impl Store {
    fn new() -> Self {
        Store {
            questions: Self::init(),
        }
    }

    fn init() -> HashMap<QuestionId, Question> {
        let file = include_str!("../../questions.json");
        serde_json::from_str(file).expect("Can't read questions")
    }
}

#[derive(Debug)]
struct InvalidId;
impl Reject for InvalidId {}

#[derive(Debug)]
enum Error {
    ParseError(std::num::ParseIntError),
    MissingParameter(String),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            Error::ParseError(ref err) => {
                write!(f, "Cannot parse parameter: {}", err)
            }
            Error::MissingParameter(ref param) => {
                write!(f, "Missing parameter: '{}'", param)
            }
        }
    }
}

impl Reject for Error {}

#[derive(Debug)]
struct Pagination {
    start: usize,
    end: usize,
}

fn extract_pagination(params: HashMap<String, String>) -> Result<Pagination, Error> {
    match (params.get("start"), params.get("end")) {
        (Some(start), Some(end)) => {
            return Ok(Pagination {
                start: start.parse::<usize>().map_err(Error::ParseError)?,
                end: end.parse::<usize>().map_err(Error::ParseError)?,
            })
        }
        (None, _) => Err(Error::MissingParameter("start".into())),
        (_, None) => Err(Error::MissingParameter("end".into())),
    }
}

async fn get_questions(
    params: HashMap<String, String>,
    store: Store,
) -> Result<impl warp::Reply, warp::Rejection> {
    if !params.is_empty() {
        let pagination = extract_pagination(params)?;
        let res: Vec<Question> = store.questions.values().cloned().collect();
        let end = pagination.end.clamp(0, store.questions.len());
        let start = pagination.start.clamp(0, end);
        let res = &res[start..end];
        Ok(warp::reply::json(&res))
    } else {
        let res: Vec<Question> = store.questions.values().cloned().collect();
        Ok(warp::reply::json(&res))
    }
}

async fn return_error(r: Rejection) -> Result<impl Reply, Rejection> {
    if let Some(error) = r.find::<CorsForbidden>() {
        Ok(warp::reply::with_status(
            error.to_string(),
            StatusCode::FORBIDDEN,
        ))
    } else if let Some(error) = r.find::<Error>() {
        Ok(warp::reply::with_status(
            error.to_string(),
            StatusCode::RANGE_NOT_SATISFIABLE,
        ))
    } else {
        if r.is_not_found() {
            Ok(warp::reply::with_status(
                "Route not found".into(),
                StatusCode::NOT_FOUND,
            ))
        } else {
            Err(r)
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let store = Store::new();
    let store_filter = warp::any().map(move || store.clone());
    let cors = warp::cors()
        .allow_any_origin()
        .allow_header("content-type")
        .allow_methods(&[Method::PUT, Method::GET, Method::POST, Method::DELETE]);
    let get_items = warp::get()
        .and(warp::path("questions"))
        .and(warp::path::end())
        .and(warp::query())
        .and(store_filter)
        .and_then(get_questions)
        .recover(return_error);

    let routes = get_items.with(cors);

    warp::serve(routes).run(([127, 0, 0, 1], 1337)).await;

    Ok(())
}
