// best practice to receive API data through a `struct`
use serde::{Deserialize, Serialize};
use reqwest_middleware::ClientBuilder;
use reqwest_retry::{RetryTransientMiddleware, policies::ExponentialBackoff};

// use handle_errors::WarpError;

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct APIResponse {
    message: String
}

#[derive(Deserialize, Serialize, Debug, Clone)]
struct BadWord {
    original: String,
    word: String,
    deviations: i64,
    info: i64,
    #[serde(rename = "replacedLen")]
    replaced_len: i64,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
struct BadWordsResponse {
    content: String,
    bad_words_total: i64,
    bad_words_list: Vec<BadWord>,
    censored_content: String,
}

// filter out bad words from `String` passed in.
pub async fn check_profanity(content: String) -> Result<String, handle_errors::WarpError> {
    // retry communicating with the API incase of initial failure
    let retry_policy = ExponentialBackoff::builder().build_with_max_retries(3);
    let client = ClientBuilder::new(reqwest::Client::new())
        .with(RetryTransientMiddleware::new_with_policy(retry_policy))
        .build();
    
    let res = client
        .post("https://api.apilayer.com/bad_words?censor_character=*")
        .header("apikey", "YdeCTRJm2dGvwfyuZkTt2JlztBfFMQ2Y") // I know this shouldn't be here; just practicing
        .body(content)
        .send()
        .await
        .map_err(|e| handle_errors::WarpError::MiddlewareReqwestAPIError(e))?;

    // handle error, if client or server returns an error
    // error from handling `adding the question` is handled seperately
    if !res.status().is_success() {
        if res.status().is_client_error() {
            let err = transform_error(res).await;
            return Err(handle_errors::WarpError::ClientError(err));
        } else {
            let err = transform_error(res).await;
            return Err(handle_errors::WarpError::ServerError(err));
        }
    }

    match res.json::<BadWordsResponse>().await {
        Ok(res) => Ok(res.censored_content),
        Err(e) => Err(handle_errors::WarpError::ReqwestAPIError(e))
    }
}

async fn transform_error(res: reqwest::Response) -> handle_errors::APILayerError {
    handle_errors::APILayerError {
        status: res.status().as_u16(),
        message: res.json::<APIResponse>().await.unwrap().message,
    }
}