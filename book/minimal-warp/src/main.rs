use std::str::FromStr;
use std::sync::Arc;
use std::{collections::HashMap, fmt::Display};
use tokio::sync::RwLock;
use uuid::Uuid;

use serde::{Deserialize, Serialize};

use warp::{
    filters::body::BodyDeserializeError, filters::cors::CorsForbidden, http::Method,
    http::StatusCode, reject::Reject, Filter, Rejection, Reply,
};

#[derive(PartialEq, PartialOrd, Debug, Clone, Serialize, Deserialize)]
struct Question {
    id: QuestionId,
    title: Title,
    content: Content,
    tags: Option<Vec<Tag>>,
}

#[derive(PartialEq, PartialOrd, Debug, Clone, Serialize, Deserialize)]
struct Answer {
    id: AnswerId,
    question_id: QuestionId,
    content: Content,
}

#[derive(PartialEq, PartialOrd, Eq, Hash, Debug, Clone, Serialize, Deserialize)]
struct AnswerId(String);

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

impl FromStr for QuestionId {
    type Err = <String as FromStr>::Err;
    fn from_str(src: &str) -> Result<Self, Self::Err> {
        Ok(Self(String::from_str(src)?))
    }
}

impl FromStr for AnswerId {
    type Err = <String as FromStr>::Err;
    fn from_str(src: &str) -> Result<Self, Self::Err> {
        Ok(Self(String::from_str(src)?))
    }
}

#[derive(Clone)]
struct Store {
    questions: Arc<RwLock<HashMap<QuestionId, Question>>>,
    answers: Arc<RwLock<HashMap<AnswerId, Answer>>>,
}

impl Store {
    fn new() -> Self {
        Store {
            questions: Arc::new(RwLock::new(Self::init_questions())),
            answers: Arc::new(RwLock::new(Self::init_answers())),
        }
    }

    fn init_questions() -> HashMap<QuestionId, Question> {
        let file = include_str!("../../questions.json");
        serde_json::from_str(file).expect("Can't read questions")
    }

    fn init_answers() -> HashMap<AnswerId, Answer> {
        let file = include_str!("../../answers.json");
        serde_json::from_str(file).expect("Can't read answers")
    }
}

#[derive(Debug)]
enum Error {
    ParseError(std::num::ParseIntError),
    MissingParameter(String),
    QuestionNotFound,
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
            Error::QuestionNotFound => {
                write!(f, "Question not found")
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
    let res: Vec<Question> = store.questions.read().await.values().cloned().collect();
    if !params.is_empty() {
        let pagination = extract_pagination(params)?;
        let end = pagination.end.clamp(0, res.len());
        let start = pagination.start.clamp(0, end);
        let res = &res[start..end];
        Ok(warp::reply::json(&res))
    } else {
        Ok(warp::reply::json(&res))
    }
}

async fn add_question(
    store: Store,
    question: Question,
) -> Result<impl warp::Reply, warp::Rejection> {
    store
        .questions
        .write()
        .await
        .insert(question.id.clone(), question);
    Ok(warp::reply::with_status("Question added", StatusCode::OK))
}

async fn add_answer(
    question_id: QuestionId,
    store: Store,
    params: HashMap<String, String>,
) -> Result<impl warp::Reply, warp::Rejection> {
    if !store.questions.read().await.contains_key(&question_id) {
        return Err(warp::reject::custom(Error::QuestionNotFound));
    }

    if let Some(content) = params.get("content") {
        let answer = Answer {
            id: AnswerId(Uuid::new_v4().to_string()),
            question_id,
            content: Content(content.to_owned()),
        };
        store
            .answers
            .write()
            .await
            .insert(answer.id.clone(), answer);
        Ok(warp::reply::with_status("Answer added", StatusCode::OK))
    } else {
        Err(warp::reject::custom(Error::MissingParameter(
            "content".into(),
        )))
    }
}

async fn update_question(
    id: QuestionId,
    store: Store,
    question: Question,
) -> Result<impl warp::Reply, warp::Rejection> {
    if let Some(q) = store.questions.write().await.get_mut(&id) {
        *q = question;
        Ok(warp::reply::with_status("Question updated", StatusCode::OK))
    } else {
        Err(warp::reject::custom(Error::QuestionNotFound))
    }
}

async fn delete_question(
    id: QuestionId,
    store: Store,
) -> Result<impl warp::Reply, warp::Rejection> {
    match store.questions.write().await.remove(&id) {
        Some(_) => Ok(warp::reply::with_status("Question removed", StatusCode::OK)),
        None => Err(warp::reject::custom(Error::QuestionNotFound)),
    }
}

async fn return_error(r: Rejection) -> Result<impl Reply, Rejection> {
    if let Some(error) = r.find::<CorsForbidden>() {
        Ok(warp::reply::with_status(
            error.to_string(),
            StatusCode::FORBIDDEN,
        ))
    } else if let Some(error) = r.find::<BodyDeserializeError>() {
        Ok(warp::reply::with_status(
            error.to_string(),
            StatusCode::UNPROCESSABLE_ENTITY,
        ))
    } else if let Some(error) = r.find::<Error>() {
        Ok(warp::reply::with_status(
            error.to_string(),
            StatusCode::RANGE_NOT_SATISFIABLE,
        ))
    } else {
        Ok(warp::reply::with_status(
            "Route not found".into(),
            StatusCode::NOT_FOUND,
        ))
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

    let get_questions = warp::get()
        .and(warp::path("questions"))
        .and(warp::path::end())
        .and(warp::query())
        .and(store_filter.clone())
        .and_then(get_questions);

    let add_question = warp::post()
        .and(warp::path("questions"))
        .and(warp::path::end())
        .and(store_filter.clone())
        .and(warp::body::json())
        .and_then(add_question);

    let add_answer = warp::post()
        .and(warp::path("questions"))
        .and(warp::path::param::<QuestionId>())
        .and(warp::path("answers"))
        .and(warp::path::end())
        .and(store_filter.clone())
        .and(warp::body::form())
        .and_then(add_answer);

    let update_question = warp::put()
        .and(warp::path("questions"))
        .and(warp::path::param::<QuestionId>())
        .and(warp::path::end())
        .and(store_filter.clone())
        .and(warp::body::json())
        .and_then(update_question);

    let delete_question = warp::delete()
        .and(warp::path("questions"))
        .and(warp::path::param::<QuestionId>())
        .and(warp::path::end())
        .and(store_filter.clone())
        .and_then(delete_question);

    let routes = get_questions
        .or(add_question)
        .or(add_answer)
        .or(update_question)
        .or(delete_question)
        .with(cors)
        .recover(return_error);

    warp::serve(routes).run(([127, 0, 0, 1], 1337)).await;

    Ok(())
}
