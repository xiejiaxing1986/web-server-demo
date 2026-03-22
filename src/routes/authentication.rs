use std::{future, env};

use crate::store::Store;
use crate::types::account::{Account, AccountId, Session};
use warp::Filter;
use warp::{Reply, Rejection, http::StatusCode};
use tracing::{instrument, event, Level};
use argon2::{self, Config};
use rand::Rng;
use rusty_paseto::prelude::{
    CustomClaim, ExpirationClaim, Key, NotBeforeClaim, PasetoBuilder, PasetoParser,
    PasetoSymmetricKey, V4,
};
use rusty_paseto::core::Local as PasetoLocal;
use chrono::prelude::*;

// const PASETO_KEY: &[u8; 32] = b"RANDOM WORDS WINTER MACINTOSH PC";

#[instrument(skip_all)]
pub async fn register(store: Store, account: Account) -> Result<impl Reply, Rejection> {
    event!(target: "web_server_demo", Level::INFO, "adding an account");

    let hashed_password = hash_password(account.password.as_bytes());
    let account = Account {
        id: account.id,
        email: account.email,
        password: hashed_password,
    };
    match store.add_account(account).await {
        Ok(_) => Ok(warp::reply::with_status("Account added", StatusCode::OK)),
        Err(e) => Err(warp::reject::custom(e)),
    }
}

pub fn hash_password(password: &[u8]) -> String {
    let salt = rand::thread_rng().r#gen::<[u8; 32]>(); //由于gen在新版本中成为保留关键字，需要加上r#以区分
    let config = Config::default();
    argon2::hash_encoded(password, &salt, &config).unwrap()
}

#[instrument(skip_all)]
pub async fn login(store: Store, login: Account) -> Result<impl Reply, Rejection> {
    match store.get_account(login.email).await {
        Ok(account) => match verify_password(
        &account.password, login.password.as_bytes()) {
            Ok(verified) => {
                if verified {
                    Ok(warp::reply::json(&issue_token(account.id.expect("id not found"))))
                } else {
                    Err(warp::reject::custom(handle_errors::Error::WrongPassword))
                }
            },
            Err(e) => Err(warp::reject::custom(handle_errors::Error::ArgonLibraryError(e)))
        },
        Err(e) => Err(warp::reject::custom(e))
    }
}

fn verify_password(hash: &str, password: &[u8]) -> Result<bool, argon2::Error> {
    argon2::verify_encoded(hash, password)
}

fn make_key() -> PasetoSymmetricKey<V4, PasetoLocal> {
    let key_str = env::var("PASETO_KEY").unwrap();
    let key_bytes = key_str.as_bytes();
    let mut key_array = [0u8; 32];
    let len = key_bytes.len().min(32);
    key_array[..len].copy_from_slice(&key_bytes[..len]);
    PasetoSymmetricKey::<V4, PasetoLocal>::from(Key::from(&key_array))
}
fn issue_token(account_id: AccountId) -> String {
    let key = make_key();
    let exp = (Utc::now() + chrono::Duration::hours(1)).to_rfc3339();
    let nbf = Utc::now().to_rfc3339();

    PasetoBuilder::<V4, PasetoLocal>::default()
        .set_claim(ExpirationClaim::try_from(exp.as_str()).unwrap())
        .set_claim(NotBeforeClaim::try_from(nbf.as_str()).unwrap())
        .set_claim(CustomClaim::try_from(("account_id", serde_json::json!(account_id))).unwrap())
        .build(&key)
        .expect("Failed to construct paseto token w/ builder!")
    
}

pub fn verify_token(token: String) -> Result<Session, handle_errors::Error> {
    let key = make_key();

    let json_value = PasetoParser::<V4, PasetoLocal>::default()
        .parse(&token, &key)
        .map_err(|_| handle_errors::Error::CannotDecryptToken)?;

    serde_json::from_value::<Session>(json_value)
        .map_err(|_| handle_errors::Error::CannotDecryptToken)
}
pub fn auth() -> impl Filter<Extract = (Session,), Error = warp::Rejection> + Clone {
    warp::header::<String>("Authorization").and_then(|token: String| {
        let token = match verify_token(token) {
            Ok(t) => t,
            Err(_) => return future::ready(Err(warp::reject::reject()))
        };

        future::ready(Ok(token))
    })
}