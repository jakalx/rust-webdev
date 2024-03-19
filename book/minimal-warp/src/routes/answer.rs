use crate::error;
use crate::Answer;
use crate::AnswerId;
use crate::Content;
use crate::QuestionId;
use crate::Store;

use std::collections::HashMap;
use uuid::Uuid;
use warp::http::StatusCode;

pub async fn add_answer(
    question_id: QuestionId,
    store: Store,
    params: HashMap<String, String>,
) -> Result<impl warp::Reply, warp::Rejection> {
    if !store.questions.read().await.contains_key(&question_id) {
        return Err(warp::reject::custom(error::Error::QuestionNotFound));
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
        Err(warp::reject::custom(error::Error::MissingParameter(
            "content".into(),
        )))
    }
}
