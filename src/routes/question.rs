use std::collections::HashMap;
use warp::{http::StatusCode, Rejection, Reply};
use tracing::{event, instrument, Level};

use crate::types::account::Session;
use crate::store::Store;
use crate::types::pagination;
use crate::types::question::{Question, NewQuestion};
use crate::profanity::check_profanity;


#[instrument]
pub async fn get_question(params: HashMap<String, String> , store: Store) -> Result<impl Reply, Rejection> {
    event!(target: "webapp_api", Level::INFO, "querying questions");
    let mut pagination = pagination::Pagination::default();
    if !params.is_empty() {
        event!(Level::INFO, pagination = true);
        pagination = pagination::get_pagination(params)?; // set Pagination value if not empty
    }
    match store.get_questions(pagination.limit, pagination.offset).await {
        Ok(res) => Ok(warp::reply::json(&res)),
        Err(e) => Err(warp::reject::custom(e)),
    }
}

// NB: the order of the arguments also matter when passing it into the main function
pub async fn add_question(session: Session, store: Store, new_question: NewQuestion) -> Result<impl Reply, Rejection> {
    let account_id = session.account_id;
    let title = match check_profanity(new_question.title).await {
        Ok(res) => res,
        Err(e) => return Err(warp::reject::custom(e)),
    };
    let content = match check_profanity(new_question.content).await {
        Ok(res) => res,
        Err(e) => return Err(warp::reject::custom(e)),
    };

    let question = NewQuestion {
        title,
        content,
        tags: new_question.tags,
    };

    match store.add_question(question, account_id).await {
        Ok(question) => Ok(warp::reply::json(&question)),
        Err(e) => Err(warp::reject::custom(e)),
    }
}

pub async fn update_question(id: i32, session: Session, store: Store, question: Question) -> Result<impl Reply, Rejection> {
    // get the `account_id` out of the `session_id` to be able to pass a reference to later functions
    let account_id = session.account_id;
    // this store function checks if the original question was created by the same account
    if store.is_question_owner(id, &account_id).await? {
        let title = check_profanity(question.title);
        let content = check_profanity(question.content);

        // run the API requests concurrently using `tokio::join`
        let (title, content) = tokio::join!(title, content);
        if title.is_ok() && content.is_ok() {
            let question = Question {
                id: question.id,
                title: title.unwrap(),
                content: content.unwrap(),
                tags: question.tags
            };
            match store.update_question(question, id, account_id).await {
                Ok(res) => Ok(warp::reply::json(&res)),
                Err(e) => Err(warp::reject::custom(e))
            }
        } else {
            Err(warp::reject::custom(title.expect_err("Expected API call to have failed here.")))
        }
    } else {
        // returns this error if the ID in the DB != ID in the session
        Err(warp::reject::custom(handle_errors::WarpError::Unauthorized))
    }
}

pub async fn delete_question(id: i32, session: Session, store: Store) -> Result<impl Reply, Rejection> {
    let account_id = session.account_id;
    if store.is_question_owner(id, &account_id).await? {
        match store.delete_question(id).await {
            Ok(_) => Ok(warp::reply::with_status(format!("Question {} deleted", id), StatusCode::OK)),
            Err(e) => Err(warp::reject::custom(e))
        }
    } else {
        Err(warp::reject::custom(handle_errors::WarpError::Unauthorized))
    }
    
}