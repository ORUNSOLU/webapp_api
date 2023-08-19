use std::collections::HashMap;
use warp::{http::StatusCode, Rejection, Reply};
use tracing::{event, instrument, Level};

use crate::store::Store;
use crate::types::pagination;
use crate::types::question::{Question, NewQuestion};


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
    match store.add_question(new_question).await {
        Ok(_) => Ok(warp::reply::with_status("Question added!", StatusCode::OK)),
        Err(e) => Err(warp::reject::custom(e)),
    }
}

pub async fn update_question(id: i32, store: Store, question: Question) -> Result<impl Reply, Rejection> {
    let res = match store.update_question(question, id).await {
        Ok(res) => res,
        Err(e) => Err(warp::reject::custom(e))
    };
    Ok(warp::reply::json(&res))
}

pub async fn delete_question(id: i32, store: Store) -> Result<impl Reply, Rejection> {
    match store.delete_question(id).await {
        Ok(_) => Ok(warp::reply::with_status(format!("Question {} deleted", id), StatusCode::OK)),
        Err(e) => Err(warp::reject::custom(e))
    }
}