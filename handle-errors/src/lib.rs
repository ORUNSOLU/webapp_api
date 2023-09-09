use warp::{
    reject::Reject, http::StatusCode, Rejection, Reply,
    filters::{
        body::BodyDeserializeError, cors::CorsForbidden
    }
};
use tracing::{event, Level, instrument};

// custom errors to map values to
#[derive(Debug)]
pub enum WarpError {
    ParseError(std::num::ParseIntError),
    MissingParameters,
    DatabaseQueryError,
}

// implement display for the WarpErrors
impl std::fmt::Display for WarpError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &*self { 
            Self::ParseError(ref err) => write!(f, "WarpError parsing parameter: {}", err),
            Self::MissingParameters => write!(f, "Some parameters are missing"),
            Self::DatabaseQueryError => write!(f, "Cannot update, invalid Data.") 
        }
    }
}

// implement *Reject for our custom-error so we can call them in *warp
// this is a prerequisite to use the `warp` error trait
impl Reject for WarpError {}

// map the different possible error types, and handle them here
#[instrument]
pub async fn return_error(r: Rejection) -> Result<impl Reply, Rejection> {
    if let Some(crate::WarpError::DatabaseQueryError) = r.find() {
        event!(Level::ERROR, "Database query error");
        Ok(warp::reply::with_status(crate::WarpError::DatabaseQueryError.to_string(),StatusCode::UNPROCESSABLE_ENTITY))
    } else if let Some(error) = r.find::<CorsForbidden>() {
        event!(Level::ERROR, "CORS forbidden error: {}", error);
        Ok(warp::reply::with_status(error.to_string(),StatusCode::FORBIDDEN))
    } else if let Some(error) = r.find::<BodyDeserializeError>() {
        event!(Level::ERROR, "Cannot deserizalize request body: {}", error);
        Ok(warp::reply::with_status(error.to_string(),StatusCode::UNPROCESSABLE_ENTITY))
    } else if let Some(error) = r.find::<WarpError>() {
        event!(Level::ERROR, "{}", error);
        Ok(warp::reply::with_status(error.to_string(),StatusCode::UNPROCESSABLE_ENTITY,)) 
    } else {
        event!(Level::WARN, "Requested route was not found");
        Ok(warp::reply::with_status("Route not found".to_string(),StatusCode::NOT_FOUND,))
    }
}