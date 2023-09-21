use warp::{http::Method, Filter};
use handle_errors::return_error; // internal library
use crate::store::Store;
use crate::routes::question::{get_question, add_question, update_question, delete_question};
use crate::routes::authentication;
use crate::routes::answer::add_answer;
use tracing_subscriber::fmt::format::FmtSpan;
// use types::*;

mod store;
mod routes;
mod types;
mod profanity;

#[tokio::main]
async fn main() {
    // environment variable to filter logs
    let log_filter = std::env::var("RUST_LOG").unwrap_or_else(|_| { 
        "handle_errors=warn,rust_training=info ,warp=error".to_owned()
    });

    // we would use the connection string below if we were to include `username & password`
    // "postgres://username:password@localhost:5432/db_name"
    let store = Store::new("postgres://postgres:postgres@localhost:5432/rustwebdev").await;
    sqlx::migrate!().run(&store.clone().conn).await.expect("Unable to run the migrations."); // code to run the migration file

    let store_filter = warp::any().map(move || store.clone());

    // tracing_subscriber::fmt()
    //     .with_env_filter(log_filter) // use the filter above to determine traces to log
    //     .with_span_events(FmtSpan::CLOSE) // records events when each span closes
    //     .init();

    tracing_subscriber::fmt().with_env_filter(log_filter).with_span_events(FmtSpan::CLOSE).init();

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
        .and_then(get_question)
        .with(warp::trace(|info| {
            tracing::info_span!(
                "get_questions request",
                method = %info.method(),
                path = %info.path(),
                uuid = %uuid::Uuid::new_v4()
            )
        }));

    let login = warp::post()
        .and(warp::path("login"))
        .and(warp::path::end())
        .and(store_filter.clone())
        .and(warp::body::json())
        .and_then(routes::authentication::login);

    let add_question = warp::post()
        .and(warp::path("questions"))
        .and(warp::path::end())
        .and(store_filter.clone())
        .and(warp::body::json())
        .and_then(add_question);

    let update_question = warp::put()
        .and(warp::path("questions"))
        .and(warp::path::param::<i32>())
        .and(warp::path::end()) // closes the path definition
        .and(store_filter.clone()) // adds our store to the route so we can pass it to the route handler later
        .and(warp::body::json()) // extracts the JSON body that's added to the parameters as well
        .and_then(update_question);

    let delete_question = warp::delete()
        .and(warp::path("questions"))
        .and(warp::path::param::<i32>())
        .and(warp::path::end())
        .and(store_filter.clone())
        .and_then(delete_question);

    let add_answer = warp::post()
        .and(warp::path("answers"))
        .and(warp::path::end())
        .and(store_filter.clone())
        .and(warp::body::form()) // this uses *url-form encoded, instead of JSON
        .and_then(add_answer);

    let registration = warp::post()
        .and(warp::path("registration"))
        .and(warp::path::end())
        .and(store_filter.clone())
        .and(warp::body::json())
        .and_then(authentication::register);

    let routes = get_questions
        .or(add_question)
        .or(update_question)
        .or(add_answer)
        .or(delete_question)
        .or(registration)
        .or(login)
        .with(cors)
        .with(warp::trace::request()) // setup logging for incoming request
        .recover(return_error);

    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}