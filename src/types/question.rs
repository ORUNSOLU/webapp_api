use serde::{Serialize,Deserialize};


// database creation structure
// CREATE TABLE IF NOT EXISTS questions (
// id serial PRIMARY KEY,
// title VARCHAR (255) NOT NULL,
// content TEXT NOT NULL,
// tags TEXT [],
// created_on TIMESTAMP NOT NULL DEFAULT NOW()
// );

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Question {
    pub id: QuestionId,
    pub title: String,
    pub content: String,
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Eq, Hash, Clone, PartialEq, Deserialize)]
pub struct QuestionId(pub i32);

// the `ID is automatically created by the DB; Check the DB definition at the top
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NewQuestion {
    pub title: String,
    pub content: String,
    pub tags: Option<Vec<String>>
}