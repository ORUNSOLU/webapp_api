use serde::{Serialize,Deserialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Question {
    pub id: QuestionId,
    pub title: String,
    pub content: String,
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Eq, Hash, Clone, PartialEq, Deserialize)]
pub struct QuestionId(pub i32);