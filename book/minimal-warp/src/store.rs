use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::types::answer::{Answer, AnswerId};
use crate::types::question::{Question, QuestionId};

#[derive(Clone)]
pub struct Store {
    pub questions: Arc<RwLock<HashMap<QuestionId, Question>>>,
    pub answers: Arc<RwLock<HashMap<AnswerId, Answer>>>,
}

impl Store {
    pub fn new() -> Self {
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
