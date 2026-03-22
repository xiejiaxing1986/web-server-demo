use serde::{Deserialize, Serialize};
use reqwest_middleware::ClientBuilder;
use reqwest_retry::{RetryTransientMiddleware, policies::ExponentialBackoff};
// use std::time::Duration;
use std::env;

#[derive(Deserialize, Serialize, Debug, Clone)] 
struct APIResponse { 
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

pub async fn check_profanity(content: String) -> Result<String, handle_errors::Error> {

    // 模拟5s的网络延迟
    // tokio::time::sleep(Duration::from_millis(5000)).await;

    let api_key = env::var("BAD_WORDS_API_KEY").unwrap();
    let retry_policy = ExponentialBackoff::builder().build_with_max_retries(3);
    let client = ClientBuilder::new(reqwest::Client::new())
        .with(RetryTransientMiddleware::new_with_policy(retry_policy))
        .build();
    let res = client
        .post("https://api.apilayer.com/bad_words?censor_character=*")
        .header("apikey", api_key)
        .body(content)
        .send()
        .await
        .map_err(|e| handle_errors::Error::MiddlewareReqwestAPIError(e))?;

    if !res.status().is_success() {
        if res.status().is_client_error() {
            let err = transform_error(res).await;
            return Err(handle_errors::Error::ClientError(err).into());
        }else {
            let err = transform_error(res).await;
            return Err(handle_errors::Error::ServerError(err).into());
        }
    }
    
    match res.json::<BadWordsResponse>().await {
        Ok(res) => Ok(res.censored_content),
        Err(e) => Err(handle_errors::Error::ReqwestAPIError(e)),
    }       
}


async fn transform_error(res: reqwest::Response) -> handle_errors::APILayerError {
    handle_errors::APILayerError {
        status: res.status().as_u16(),
        message: res.json::<APIResponse>().await.unwrap().message,
    }
}