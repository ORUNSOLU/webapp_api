// this module should handle all DB connections for all routes
use sqlx::postgres::{PgPoolOptions, PgPool, PgRow};
use sqlx::Row;
use handle_errors::WarpError; // internal Library

use crate::types::{
    answer::{Answer, AnswerId, NewAnswer},
    question::{QuestionId, Question, NewQuestion},
    account::{Account, AccountId}
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
                Err(e) => panic!("Could not establish database connection: {}", e),
            };
        Self {
            conn: db_pool
        }
    }

    // `offset` indicates where to start querying;
    // `limit` gives us the number of result we want
    // offset = no to start questions from e.g. 50;; limit = no of questions to get e.g. 10
    // if offset =50, limit=10....questions returned will be from 50 + 10 = questions 50 - 59
    pub async fn get_questions(&self, limit: Option<i32>, offset: i32) -> Result<Vec<Question>, WarpError> {
        match sqlx::query("SELECT * from questions LIMIT $1 OFFSET $2")
            .bind(limit)
            .bind(offset)
            .map(|row: PgRow| Question { // use MAP to get rows returned by PG, and create a Question from it
                id: QuestionId(row.get("id")),
                title: row.get("title"),
                content: row.get("content"),
                tags: row.get("tags"),
            })
            .fetch_all(&self.conn)
            .await {
                Ok(questions) => Ok(questions),
                Err(e) => {
                    tracing::event!(tracing::Level::ERROR, "{:?}", e);
                    Err(WarpError::DatabaseQueryError(e))
                }
        }
    }

    pub async fn add_question(self, new_question: NewQuestion, account_id: AccountId) -> Result<Question, WarpError> {
        match sqlx::query("INSERT INTO questions (title, content, tags, account_id)
                            VALUES ($1, $2, $3, $4)
                            RETURNING id, title, content, tags"
            )
            .bind(new_question.title)
            .bind(new_question.content)
            .bind(new_question.tags)
            .bind(account_id.0)
            .map(|row: PgRow| Question {
                id: QuestionId(row.get("id")),
                title: row.get("title"),
                content: row.get("content"),
                tags: row.get("tags")
            })
            .fetch_one(&self.conn)
            .await {
                Ok(question) => Ok(question),
                Err(e) => { 
                    tracing::event!(tracing::Level::ERROR, "{:?}", e);
                    Err(WarpError::DatabaseQueryError(e)) 
                }
            }
    }

    pub async fn update_question(&self, question: Question, question_id: i32, account_id: AccountId) -> Result<Question, WarpError> {
        println!("Account ID: {:?}", account_id.0);
        match sqlx::query("UPDATE questions
                            SET title = $1, content = $2, tags = $3
                            WHERE id = $4 AND account_id = $5
                            RETURNING id, title, content, tags
        ")
        .bind(question.title)
        .bind(question.content)
        .bind(question.tags)
        .bind(question_id)
        .bind(account_id.0)
        .map(|row: PgRow| Question {
            id: QuestionId(row.get("id")),
            title: row.get("title"),
            content: row.get("content"),
            tags: row.get("tags"),
        })
        .fetch_one(&self.conn)
        .await {
            Ok(question) => Ok(question),
            Err(e) => {
                tracing::event!(tracing::Level::ERROR, "{:?}", e);
                Err(WarpError::DatabaseQueryError(e))
            }
        }
    }

    pub async fn delete_question(&self, question_id: i32) -> Result<bool, WarpError> {
        match sqlx::query("DELETE FROM questions where id = $1")
            .bind(question_id)
            .execute(&self.conn) // use `execute` from sqlx since we cannot return a deleted row
            .await {
                Ok(_) => Ok(true),
                Err(e) => { 
                    tracing::event!(tracing::Level::ERROR, "{:?}", e);
                    Err(WarpError::DatabaseQueryError(e)) 
                }
            }
    }

    pub async fn add_answer(&self, new_answer: NewAnswer, account_id: AccountId) -> Result<Answer, WarpError> {
        match sqlx::query(
                "INSERT INTO answers (content, corresponding_question, account_id) VALUES ($1, $2, $3)"
            )
            .bind(new_answer.content)
            .bind(new_answer.question_id.0)
            .bind(account_id.0)
            .map(|row: PgRow| Answer {
                id: AnswerId(row.get("id")),
                content: row.get("content"),
                question_id: QuestionId(row.get("question_id")),
            })
            .fetch_one(&self.conn)
            .await {
                Ok(answer) => Ok(answer),
                Err(e) => {
                    tracing::event!(tracing::Level::ERROR, "{:?}", e);
                    Err(WarpError::DatabaseQueryError(e))
                },
            }
    }

    pub async fn add_account(self, account: Account) -> Result<bool, WarpError> {
        match sqlx::query(
            "INSERT INTO accounts (email, password) VALUES ($1, $2)"
        )
        .bind(account.email)
        .bind(account.password)
        .execute(&self.conn)
        .await {
            Ok(_) => Ok(true),
            Err(error) => {
                tracing::event!(
                    tracing::Level::ERROR,
                    code = error.as_database_error().unwrap().code().unwrap().parse::<i32>().unwrap(),
                    db_message = error.as_database_error().unwrap().message(),
                    constraint = error.as_database_error().unwrap().constraint().unwrap()
                );
                Err(WarpError::DatabaseQueryError(error))
            }
        }
    }

    pub async fn get_account(self, email: String) -> Result<Account, WarpError> {
        match sqlx::query("SELECT * FROM accounts WHERE email = $1")
            .bind(email)
            .map(|row: PgRow| Account {
                id: Some(AccountId(row.get("id"))),
                email: row.get("email"),
                password: row.get("password")
            })
            .fetch_one(&self.conn)
            .await {
                Ok(account) => Ok(account),
                Err(error) => {
                    tracing::event!(tracing::Level::ERROR, "{}", error);
                    Err(WarpError::DatabaseQueryError(error))
                }
            }
    }

    pub async fn is_question_owner(&self, question_id: i32, account_id: &AccountId) -> Result<bool, WarpError> {
        match sqlx::query("SELECT * from questions where id = $1 and account_id = $2")
            .bind(question_id)
            .bind(account_id.0)
            .fetch_optional(&self.conn)
            .await {
                Ok(question) => Ok(question.is_some()),
                Err(e) => {
                    tracing::event!(tracing::Level::ERROR, "{:?}", e);
                    Err(WarpError::DatabaseQueryError(e))
                }
            }
    }
}