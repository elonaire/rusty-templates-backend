use axum::{
    extract::{Extension, Json},
    http::HeaderMap,
    response::IntoResponse,
};
use std::sync::Arc;
use surrealdb::{engine::remote::ws::Client, Surreal};
use crate::graphql::schemas::paystack::ChargeEvent;
use hmac::{Hmac, Mac};
use sha2::Sha512;
use dotenvy::dotenv;
use std::env;
use hex;

// Type alias for HMAC-SHA512
type HmacSha512 = Hmac<Sha512>;

// Utility function to get the secret key
fn get_secret_key() -> String {
    dotenv().ok();
    env::var("PAYSTACK_SECRET").expect("PAYSTACK_SECRET must be set")
}

// The actual handler
pub async fn handle_paystack_webhook(
    Extension(db): Extension<Arc<Surreal<Client>>>,
    headers: HeaderMap,
    Json(body): Json<ChargeEvent>,
) -> impl IntoResponse {
    println!("Body: {:?}", body);

    // Get the secret key
    let secret = get_secret_key();

    // Retrieve the x-paystack-signature header
    let signature = headers.get("x-paystack-signature").and_then(|v| v.to_str().ok()).unwrap_or("");

    // Verify the webhook payload
    let mut mac = HmacSha512::new_from_slice(secret.as_bytes())
        .expect("HMAC can take key of any size");
    mac.update(serde_json::to_string(&body).expect("Failed to serialize body").as_bytes());
    let result = mac.finalize();
    let hash = hex::encode(result.into_bytes());

    if hash == signature {
        if body.event == "charge.success".to_string() {
            println!("Charge Success Body: {:?}", body);
        }
        Json(true)
    } else {
        Json(false)
    }
}
