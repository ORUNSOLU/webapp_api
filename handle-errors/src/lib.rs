// pub fn add(left: usize, right: usize) -> usize {
//     left + right
// }

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn it_works() {
//         let result = add(2, 2);
//         assert_eq!(result, 4);
//     }
// }

use warp::{
    reject::Reject, http::StatusCode, Rejection, Reply,
    filters::{
        body::BodyDeserializeError, cors::CorsForbidden
    }
};
use sqlx::error::Error as SqlxError;


#[derive(Debug)]
pub enum Error {
    ParseError(std::num::ParseIntError),
    MissingParameters,
    QuestionNotFound,
    DatabaseQueryError(SqlxError),
}

// implement display for the Errors
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self { 
            Self::ParseError(ref err) => write!(f, "Error parsing parameter: {}", err),
            Self::MissingParameters => write!(f, "Some parameters are missing"),
            Self::QuestionNotFound => write!(f, "Question not found"),
            Self::DatabaseQueryError(e) => { 
                write!(f, "Query could not be executed", e) 
            }
        }
    }
}

// implement *Reject for our custom-error so we can call them in *warp
impl Reject for Error {}

// map the different possible error types
pub async fn return_error(r: Rejection) -> Result<impl Reply, Rejection> {
    if let Some(err) = r.find::<Error>() {
        Ok(warp::reply::with_status(err.to_string(), StatusCode::RANGE_NOT_SATISFIABLE))
    } else if let Some(error) = r.find::<CorsForbidden>() {
        Ok(warp::reply::with_status(error.to_string(), StatusCode::FORBIDDEN))
    } else if let Some(err) = r.find::<BodyDeserializeError>() {
        Ok(warp::reply::with_status(err.to_string(), StatusCode::UNPROCESSABLE_ENTITY))
    } else {
        Ok(warp::reply::with_status("Route not found".to_string(), StatusCode::NOT_FOUND))
    }
}