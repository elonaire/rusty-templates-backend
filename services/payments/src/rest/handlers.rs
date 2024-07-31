use axum::{
    extract::{Extension, Json},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
};
use hyper::header::COOKIE;
use lib::{integration::{auth::internal_sign_in, email::send_email, order::update_order}, utils::models::{Email, EmailUser, OrderStatus}};
use serde_json::Value;
use std::{sync::Arc, env};
use surrealdb::{engine::remote::ws::Client, Surreal};
use hmac::{Hmac, Mac};
use sha2::Sha512;
use hex;

// Type alias for HMAC-SHA512
type HmacSha512 = Hmac<Sha512>;

pub async fn handle_paystack_webhook(
    Extension(_db): Extension<Arc<Surreal<Client>>>,
    headers: HeaderMap,
    Json(body): Json<Value>,
) -> impl IntoResponse {
    println!("Body: {:?}", body);
    // Retrieve the x-paystack-signature header
    let signature = headers.get("x-paystack-signature").and_then(|v| v.to_str().ok()).unwrap_or("");

    // Get the secret key
    let secret = env::var("PAYSTACK_SECRET").expect("PAYSTACK_SECRET must be set");
    println!("PAYSTACK_SECRET: {}", secret);

    // Verify the webhook payload
    let mut mac = HmacSha512::new_from_slice(secret.as_bytes())
        .expect("HMAC can take key of any size");
    mac.update(serde_json::to_string(&body).expect("Failed to serialize body").as_bytes());
    let result = mac.finalize();
    let hash = hex::encode(result.into_bytes());

    if hash != signature {
        // HMAC validation passed
        if let Some(event) = body.get("event").and_then(|e| e.as_str()) {
            if event == "charge.success" {
                if let Some(data) = body.get("data") {
                    if let Some(reference) = data.get("reference").and_then(|r| r.as_str()) {
                        println!("Charge Success Body: {:?}", data);
                        if let Ok(internal_jwt) = internal_sign_in().await {
                            let mut header_map = HeaderMap::new();
                            header_map.insert("Authorization", format!("Bearer {}", &internal_jwt).as_str().parse().unwrap());
                            header_map.insert(COOKIE, format!("oauth_client=;t={}", &internal_jwt).as_str().parse().unwrap());

                            // Update order status
                            if let Err(e) = update_order(header_map.clone(), reference.to_string(), OrderStatus::Confirmed).await {
                                eprintln!("Failed to update order: {:?}", e);
                                // return (
                                //     StatusCode::BAD_REQUEST,
                                //     format!("Transaction successful but could not update order status!"),
                                // ).into_response();
                            }

                            // Construct and send confirmation email
                            let confirmed_mail = if let Some(customer) = data.get("customer") {
                                if let Some(email) = customer.get("email").and_then(|e| e.as_str()) {
                                    let email_body = format!(r#"
                                    <div style="font-family: Arial, sans-serif; background-color: #f4f4f4;">
                                        <div style="max-width: 600px; margin: auto; background-color: #ffffff; border-radius: 8px; box-shadow: 0 0 10px rgba(0, 0, 0, 0.1);">
                                            <h2 style="background-color: #4CAF50; color: #ffffff; padding: 10px; border-radius: 8px 8px 0 0; text-align: center;">Payment Confirmation</h2>
                                            <div style="padding: 10px;">
                                                <p>Dear Customer,</p>
                                                <p>We are pleased to inform you that we have successfully received your payment.</p>
                                                <p>You will receive a download link shortly.</p>
                                                <p>If you have any questions or concerns, please do not hesitate to contact our support team.</p>
                                                <p>Thank you for your purchase!</p>
                                                <p>Sincerely,<br/>The Rusty Templates Team</p>
                                            </div>
                                        </div>
                                    </div>
                                    "#
                                    );

                                    Some(Email {
                                        recipient: EmailUser {
                                            full_name: None,
                                            email_address: email.to_string(),
                                        },
                                        subject: "Payment Confirmation".to_string(),
                                        title: "Payment Received! Thanks!".to_string(),
                                        body: email_body.to_string()
                                    })
                                } else {
                                    None
                                }
                            } else {
                                None
                            };

                            if let Some(email) = confirmed_mail {
                                if let Err(e) = send_email(header_map.clone(), email).await {
                                    eprintln!("Failed to send email: {:?}", e);
                                    // return (
                                    //     StatusCode::BAD_REQUEST,
                                    //     format!("Transaction successful but could not send email!"),
                                    // ).into_response();
                                }
                            }
                        };
                    }
                }
                (StatusCode::CREATED, format!("Transaction successful!")).into_response()
            } else {
                (StatusCode::BAD_REQUEST, format!("Unhandled event type: {}", event)).into_response()
            }
        } else {
            (StatusCode::BAD_REQUEST, format!("Event type missing or invalid")).into_response()
        }
    } else {
        println!("Invalid signature: expected {}, got {}", signature, hash);
        (
            StatusCode::BAD_REQUEST,
            format!("Transaction failed!"),
        ).into_response()
    }
}
