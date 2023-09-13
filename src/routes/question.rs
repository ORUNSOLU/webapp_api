use std::collections::HashMap;
use warp::{http::StatusCode, Rejection, Reply};
use tracing::{event, instrument, Level};

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

pub async fn add_question(store: Store, new_question: NewQuestion) -> Result<impl Reply, Rejection> {
    let title = match check_profanity(new_question.title).await {
        Ok(res) => res,
        Err(e) => return Err(warp::reject::custom(e))
    };

    let content = match check_profanity(new_question.content).await {
        Ok(res) => res,
        Err(e) => return Err(warp::reject::custom(e))
    };

    let question = NewQuestion {
        title: title,
        content: content,
        tags: new_question.tags
    };

    match store.add_question(question).await {
        Ok(_) => Ok(warp::reply::with_status("Question added!", StatusCode::OK)),
        Err(e) => Err(warp::reject::custom(e)),
    }
}

pub async fn update_question(id: i32, store: Store, question: Question) -> Result<impl Reply, Rejection> {
    // let title = match check_profanity(question.title).await {
    //     Ok(res) => res,
    //     Err(e) => return Err(warp::reject::custom(e)),
    // };

    // let content = match check_profanity(question.content).await {
    //     Ok(res) => res,
    //     Err(e) => return Err(warp::reject::custom(e))
    // };

    let title = check_profanity(question.title);
    let content = check_profanity(question.content);

    // run the API requests concurrently
    let (title, content) = tokio::join!(title, content);

    if title.is_err() {
        return Err(warp::reject::custom(title.unwrap_err()));
    }

    if content.is_err() {
        return Err(warp::reject::custom(content.unwrap_err()));
    }

    let question = Question {
        id: question.id,
        title: title.unwrap(),
        content: content.unwrap(),
        tags: question.tags
    };

    match store.update_question(question, id).await {
        Ok(res) => Ok(warp::reply::json(&res)),
        Err(e) => Err(warp::reject::custom(e))
    }
}

pub async fn delete_question(id: i32, store: Store) -> Result<impl Reply, Rejection> {
    match store.delete_question(id).await {
        Ok(_) => Ok(warp::reply::with_status(format!("Question {} deleted", id), StatusCode::OK)),
        Err(e) => Err(warp::reject::custom(e))
    }
}