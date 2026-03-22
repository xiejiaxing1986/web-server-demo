use crate::store::Store;
use crate::types::pagination::{Pagination, extract_pagination};
use crate::types::question::{Question, NewQuestion};
use crate::types::account::Session;
use crate::profanity::check_profanity;
use std::collections::HashMap;
use warp::{Rejection, Reply, http::StatusCode};
use tracing::{info, instrument, event, Level};


#[instrument(skip_all)]
pub async fn get_questions(
    params: HashMap<String, String>,
    store: Store
) -> Result<impl Reply, Rejection> {
    
    event!(target: "web_server_demo", Level::INFO, "querying questions");
    let mut pagination = Pagination::default();

    if !params.is_empty() {
        event!(Level::INFO, pagination = true);
        pagination = extract_pagination(params)?;
    }else {
        info!(pagination = false);
    }

    let res = match store.get_questions(pagination.limit, pagination.offset).await {
            Ok(res) => res,
            Err(e) => {
                return Err(warp::reject::custom(e))
            },
        };

    Ok(warp::reply::json(&res))
}

#[instrument(skip_all)]
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

#[instrument(skip_all)]
pub async fn update_question(
    id: i32,
    session: Session,
    store: Store,
    question: Question,
) -> Result<impl Reply, Rejection> {
    //不使用并发
    // let title = match check_profanity(question.title).await {
    //     Ok(res) => res,
    //     Err(e) => return Err(warp::reject::custom(e)),
    // };
    // let content = match check_profanity(question.content).await {
    //     Ok(res) => res,
    //     Err(e) => return Err(warp::reject::custom(e)),
    // };
    // let question = Question {
    //     id: question.id,
    //     title,
    //     content,
    //     tags: question.tags,
    // };
    // match store.update_question(question, id).await {
    //     Ok(res) => Ok(warp::reply::json(&res)),
    //     Err(e) => Err(warp::reject::custom(e)),
    // }

    //join宏实现的单线程并发
    // let title = check_profanity(question.title);
    // let content = check_profanity(question.content);
    // let (title, content) = tokio::join!(title, content);
    // if title.is_err(){
    //     return Err(warp::reject::custom(title.unwrap_err()));
    // }
    // if content.is_err(){
    //     return Err(warp::reject::custom(content.unwrap_err()));
    // }
    // let question = Question {
    //     id: question.id,
    //     title: title.unwrap(),
    //     content: content.unwrap(),
    //     tags: question.tags,
    // };
    // match store.update_question(question, id).await {
    //     Ok(res) => Ok(warp::reply::json(&res)),
    //     Err(e) => Err(warp::reject::custom(e)),
    // }

    //tokio::spwan实现并行
    let account_id = session.account_id;
    if store.is_question_owner(id, &account_id).await? {
        let title = tokio::spawn(check_profanity(question.title));
        let content = tokio::spawn(check_profanity(question.content));
        let (title, content) = (title.await.unwrap(), content.await.unwrap());
        if title.is_err(){
            return Err(warp::reject::custom(title.unwrap_err()));
        }
        if content.is_err(){
            return Err(warp::reject::custom(content.unwrap_err()));
        }
        let question = Question {
            id: question.id,
            title: title.unwrap(),
            content: content.unwrap(),
            tags: question.tags,
        };
        match store.update_question(question, id, account_id).await {
            Ok(res) => Ok(warp::reply::json(&res)),
            Err(e) => Err(warp::reject::custom(e)),
        }
    } else {
        Err(warp::reject::custom(handle_errors::Error::Unauthorized))
    }


}

#[instrument(skip_all)]
pub async fn delete_question(id: i32, session: Session, store: Store) -> Result<impl Reply, Rejection> {
    let account_id = session.account_id;
    if store.is_question_owner(id, &account_id).await? {
        if let Err(e) = store.delete_question(id, account_id).await {
            return Err(warp::reject::custom(e));
        }

        Ok(warp::reply::with_status(format!("Question {} deleted", id), StatusCode::OK))
    } else {
        Err(warp::reject::custom(handle_errors::Error::Unauthorized))
    }

}
