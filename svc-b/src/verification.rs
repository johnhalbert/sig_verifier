use serde_derive::{Deserialize, Serialize};
use actix_web::{get, post, web, Responder};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VerificationRequest {
    payload: String,
    signature: String,
    pub_key: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VerificationResponse {
    ok: bool,
}

#[get("/account/{accountID}/sign/{transactionID}")]
async fn verification_status(path_params: web::Path<(String, String)>) -> impl Responder {
    let (account_id, transaction_id) = path_params.into_inner();
    format!("OK")
}
