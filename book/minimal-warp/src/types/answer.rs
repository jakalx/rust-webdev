use serde::{Deserialize, Serialize};
use std::str::FromStr;

use crate::types::question::{Content, QuestionId};

#[derive(PartialEq, PartialOrd, Debug, Clone, Serialize, Deserialize)]
pub struct Answer {
    pub id: AnswerId,
    pub question_id: QuestionId,
    pub content: Content,
}

#[derive(PartialEq, PartialOrd, Eq, Hash, Debug, Clone, Serialize, Deserialize)]
pub struct AnswerId(pub String);

impl FromStr for AnswerId {
    type Err = <String as FromStr>::Err;
    fn from_str(src: &str) -> Result<Self, Self::Err> {
        Ok(Self(String::from_str(src)?))
    }
}
