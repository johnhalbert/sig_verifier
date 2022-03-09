use serde_derive::{Deserialize, Serialize};
use actix_web::{get, post, web, HttpResponse, error::Error};
use crate::i_redis::Redis;
use std::sync::{Arc, Mutex};

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RegistrationRequest {
    pub_key: String
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VerificationRequest {
    payload: String,
    signature: String,
    transaction_id: Option<String>,
    pub_key: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VerificationResponse {
    transaction_id: String,
    complete: bool,
    valid: Option<bool>,
}

fn internal_server_error(_: ()) -> actix_web::error::Error {
  actix_web::error::ErrorInternalServerError("Internal Error")
}

fn accepted(payload: Option<String>) -> HttpResponse {
    let mut response = HttpResponse::Accepted();

    if payload.is_some() {
        response
            .content_type("application/json")
            .body(payload.unwrap())
    } else {
        response.finish()
    }
}

#[post("/accounts/{accountID}")]
async fn register(account_id: web::Path<String>, request_body: web::Json<RegistrationRequest>, redis: web::Data<Arc<Mutex<Redis>>>) -> Result<HttpResponse, Error> {
    let mut redis = redis.lock().unwrap();
    redis.upsert_account(account_id.into_inner(), request_body.pub_key.clone()).map_err(internal_server_error)?;
    Ok(accepted(None))
}

#[post("/accounts/{accountID}/sign/{transactionID}")]
async fn verify_signature(path_params: web::Path<(String, String)>, mut request_body: web::Json<VerificationRequest>, redis: web::Data<Arc<Mutex<Redis>>>) -> Result<HttpResponse, Error> {
    let (account_id, transaction_id) = path_params.into_inner();
    let response = VerificationResponse {
        transaction_id: transaction_id.clone(),
        complete: false,
        valid: None,
    };
    let mut redis = redis.lock().unwrap();
    let pub_key = redis.get_pub_key(account_id).map_err(internal_server_error)?;
    request_body.transaction_id = Some(transaction_id.clone());
    request_body.pub_key = Some(pub_key);
    redis.upsert_verification_request(transaction_id, response.clone()).map_err(internal_server_error)?;
    redis.push_verification_request(&request_body).map_err(internal_server_error)?;
    Ok(accepted(None))
}

#[get("/accounts/{accountID}/sign/{transactionID}")]
async fn verification_status(path_params: web::Path<(String, String)>, redis: web::Data<Arc<Mutex<Redis>>>) -> Result<HttpResponse, Error> {
    let (_, transaction_id) = path_params.into_inner();
    let mut redis = redis.lock().unwrap();
    let status = redis.get_verification_status(transaction_id).map_err(internal_server_error)?;
    Ok(accepted(Some(status)))
}
