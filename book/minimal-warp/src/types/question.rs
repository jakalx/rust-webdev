use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(PartialEq, PartialOrd, Debug, Clone, Serialize, Deserialize)]
pub struct Question {
    pub id: QuestionId,
    title: Title,
    content: Content,
    tags: Option<Vec<Tag>>,
}

#[derive(PartialEq, PartialOrd, Eq, Hash, Debug, Clone, Serialize, Deserialize)]
pub struct QuestionId(String);
#[derive(PartialEq, PartialOrd, Debug, Clone, Serialize, Deserialize)]
pub struct Title(String);
#[derive(PartialEq, PartialOrd, Debug, Clone, Serialize, Deserialize)]
pub struct Content(pub String);
#[derive(PartialEq, PartialOrd, Debug, Clone, Serialize, Deserialize)]
pub struct Tag(String);

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
