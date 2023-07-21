use warp::{Filter, http::Method, Rejection, Reply, http::StatusCode, reject::Reject,
    filters::cors::CorsForbidden, filters::body::BodyDeserializeError};
use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use std::sync::Arc;
use tokio::sync::RwLock;



#[tokio::main]
async fn main() {
    let store = Store::new();
    let store_filter = warp::any().map(move || store.clone());

    // Cross Origin
    let cors = warp::cors()
        .allow_any_origin()
        .allow_header("content-type")
        .allow_methods(&[Method::PUT, Method::DELETE, Method::GET, Method::POST]);

    let get_questions = warp::get()
        .and(warp::path("questions"))
        .and(warp::path::end())
        .and(warp::query())
        .and(store_filter.clone())
        .and_then(get_questions);

    let add_question = warp::post()
        .and(warp::path("questions"))
        .and(warp::path::end())
        .and(store_filter.clone())
        .and(warp::body::json())
        .and_then(add_question);

    let update_question = warp::put()
        .and(warp::path("questions"))
        .and(warp::path::param::<String>()) 
        .and(warp::path::end()) // closes the path definition
        .and(store_filter.clone()) // adds our store to the route so we can pass it to the route handler later
        .and(warp::body::json()) // extracts the JSON body that's added to the parameters as well
        .and_then(update_question);

    let delete_question = warp::delete()
        .and(warp::path("questions"))
        .and(warp::path::param::<String>())
        .and(warp::path::end())
        .and(store_filter.clone())
        .and_then(delete_question);

    let add_answer = warp::post()
        .and(warp::path("answers"))
        .and(warp::path::end())
        .and(store_filter.clone())
        .and(warp::body::form()) // this uses *url-form encoded, instead of JSON
        .and_then(add_answer);

    let routes = get_questions
        .or(add_question)
        .or(update_question)
        .or(add_answer)
        .or(delete_question)
        .with(cors)
        .recover(return_error);

    warp::serve(routes)
        .run(([127, 0, 0, 1], 3030))
        .await;
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Question {
    id: QuestionId,
    title: String,
    content: String,
    tags: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Eq, Hash, Clone, PartialEq, Deserialize)]
struct QuestionId(String);

#[derive(Debug)]
enum Error {
    ParseError(std::num::ParseIntError),
    MissingParameters,
    QuestionNotFound
}

// implement display for the Errors
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self { 
            Self::ParseError(ref err) => write!(f, "Error parsing parameter: {}", err),
            Self::MissingParameters => write!(f, "Some parameters are missing"),
            Self::QuestionNotFound => write!(f, "Question not found")
        }
    }
}

// implement *Reject for our custom-error so we can call them in *warp
impl Reject for Error {}

async fn get_questions(params: HashMap<String, String> , store: Store) -> Result<impl Reply, Rejection> {
    if !params.is_empty() {
        let pagination = get_pagination(params)?;
        let res: Vec<Question> = store.questions.read().await.values().cloned().collect();
        let res = &res[pagination.start..pagination.end];
        Ok(warp::reply::json(&res))
    } else {
        let res: Vec<Question> = store.questions.read().await.values().cloned().collect();
        Ok(warp::reply::json(&res))
    }
}

async fn return_error(r: Rejection) -> Result<impl Reply, Rejection> {
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

#[derive(Clone)]
struct Store {
    questions: Arc<RwLock<HashMap<QuestionId, Question>>>,
    answers: Arc<RwLock<HashMap<AnswerId, Answer>>>
}

impl Store {
    fn new() -> Self {
        Self {
            questions: Arc::new(RwLock::new(Self::init())),
            answers: Arc::new(RwLock::new(HashMap::new()))
        }
    }

    fn init() -> HashMap<QuestionId, Question> {
        let file = include_str!("../questions.json");
        serde_json::from_str(file).expect("Failed to read file.")
    }
}

// to capture the values of the *start & *end values in the URL
#[derive(Debug)]
struct Pagination {
    start: usize,
    end: usize
}

// performs checks on the Pagination struct & returns the struct if okay
fn get_pagination(params: HashMap<String, String>) -> Result<Pagination, Error> {
    if params.contains_key("start") && params.contains_key("end") {
        let start = params.get("start").unwrap().parse::<usize>().unwrap();
        let end = params.get("end").unwrap().parse::<usize>().unwrap();
        if start < end && end < params.keys().count() {
        return Ok(
            Pagination {
                start: params.get("start").unwrap().parse::<usize>().map_err(Error::ParseError)?,
                end: params.get("end").unwrap().parse::<usize>().map_err(Error::ParseError)?
            }
        );
        }
    }
    Err(Error::MissingParameters)
}

async fn add_question(store: Store, question: Question) -> Result<impl Reply, Rejection> {
    store.questions.write().await.insert(question.id.clone(), question);
    Ok(warp::reply::with_status("Question added!", StatusCode::OK))
}

async fn update_question(id: String, store: Store, question: Question) -> Result<impl Reply, Rejection> {
    match store.questions.write().await.get_mut(&QuestionId(id)) {
        Some(q) => *q = question,
        None => return Err(warp::reject::custom(Error::QuestionNotFound))
    }
    Ok(warp::reply::with_status("Question updated", StatusCode::OK))
}

async fn delete_question(id: String, store: Store) -> Result<impl Reply, Rejection> {
    match store.questions.write().await.remove(&QuestionId(id)) {
        Some(_) => return Ok(warp::reply::with_status("Question deleted!", StatusCode::OK)),
        None => return Err(warp::reject::custom(Error::QuestionNotFound))
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, Hash)]
struct AnswerId(String);

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Answer {
    id: AnswerId,
    content: String,
    question_id: QuestionId
}

async fn add_answer(store: Store, params: HashMap<String, String>) -> Result<impl Reply, Rejection> {
    let answer = Answer {
        id: AnswerId("1".to_string()),
        content: params.get("content").unwrap().to_string(),
        question_id: QuestionId(params.get("questionId").unwrap().to_string())
    };
    store.answers.write().await.insert(answer.id.clone(), answer);
    Ok(warp::reply::with_status("Answer added", StatusCode::OK))
}