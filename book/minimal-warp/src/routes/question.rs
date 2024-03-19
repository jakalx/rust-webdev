use crate::types::pagination;
use crate::types::question::{Question, QuestionId};
use crate::{error, Store};
use std::collections::HashMap;
use warp::http::StatusCode;

pub async fn get_questions(
    params: HashMap<String, String>,
    store: Store,
) -> Result<impl warp::Reply, warp::Rejection> {
    let res: Vec<Question> = store.questions.read().await.values().cloned().collect();
    if !params.is_empty() {
        let pagination = pagination::extract_pagination(params)?;
        let end = pagination.end.clamp(0, res.len());
        let start = pagination.start.clamp(0, end);
        let res = &res[start..end];
        Ok(warp::reply::json(&res))
    } else {
        Ok(warp::reply::json(&res))
    }
}

pub async fn add_question(
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

pub async fn update_question(
    id: QuestionId,
    store: Store,
    question: Question,
) -> Result<impl warp::Reply, warp::Rejection> {
    match store.questions.write().await.get_mut(&id) {
        Some(q) => {
            *q = question;
            Ok(warp::reply::with_status("Question updated", StatusCode::OK))
        }
        None => Err(warp::reject::custom(error::Error::QuestionNotFound)),
    }
}

pub async fn delete_question(
    id: QuestionId,
    store: Store,
) -> Result<impl warp::Reply, warp::Rejection> {
    match store.questions.write().await.remove(&id) {
        Some(_) => Ok(warp::reply::with_status("Question removed", StatusCode::OK)),
        None => Err(warp::reject::custom(error::Error::QuestionNotFound)),
    }
}
