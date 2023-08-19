use sqlx::postgres::{PgPoolOptions, PgPool, PgRow};
use sqlx::Row;
use handle_errors::Error; // internal Library

use crate::types::{
    answer::{Answer, AnswerId},
    question::{QuestionId, Question}
};

#[derive(Clone, Debug)]
pub struct Store {
    pub conn: PgPool
}

impl Store {
    pub async fn new(db_url: &str) -> Self {
        let db_pool = match PgPoolOptions::new()
            .max_connections(5)
            .connect(db_url)
            .await {
                Ok(pool) => pool,
                Err(e) => panic!("Could not establish database connection: {:?}", e),
            }
        Self {
            conn: db_pool
        }
    }

    pub async fn get_questions(&self, limit: Option<u32>, offset: u32) -> Result<Vec<Question>, sqlx::Error> {
        match sqlx::query("SELECT * FROM questions LIMIT $1 OFFSET $2")
            .bind(limit) // $X is replaced with what we pass to *bind
            .bind(offset)
            .map(|row: PgRow| Question { // use MAP to get rows returned by PG, and create a Question from it
                id: QuestionId(row.get("id")),
                title: row.get("title"),
                content: row.get("content"),
                tags: row.get("tags"),
            } )
            .fetch_all(&self.conn)
            .await {
                Ok(questions) => Ok(questions),
                Err(e) => {
                    tracing::event!(tracing::Level::ERROR, "{:?}", e);
                    Err(Error::DatabaseError)
                }
        }
    }
}