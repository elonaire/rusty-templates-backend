use axum::{
    extract::{Extension, Json},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
};
use lib::{integration::{email::send_email, order::update_order}, utils::models::{Email, EmailUser, OrderStatus}};
use std::{sync::Arc, env};
use surrealdb::{engine::remote::ws::Client, Surreal};
use crate::graphql::schemas::paystack::ChargeEvent;
use hmac::{Hmac, Mac};
use sha2::Sha512;
use hex;

// Type alias for HMAC-SHA512
type HmacSha512 = Hmac<Sha512>;

pub async fn handle_paystack_webhook(
    Extension(_db): Extension<Arc<Surreal<Client>>>,
    headers: HeaderMap,
    Json(body): Json<ChargeEvent>,
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

    if hash == signature {
        // HMAC validation passed
        if body.event == "charge.success".to_string() {
            println!("Charge Success Body: {:?}", body);

            if let Err(e) = update_order(headers.clone(), body.data.reference, OrderStatus::Confirmed).await {
                eprintln!("Failed to update order: {:?}", e);
                return (
                    StatusCode::BAD_REQUEST,
                    format!("Transaction successful but could not update order status!"),
                ).into_response()
            }

            let email_body = r#"
            <div style="font-family: Arial, sans-serif; background-color: #f4f4f4;">
              <div style="max-width: 600px; margin: auto; background-color: #ffffff; border-radius: 8px; box-shadow: 0 0 10px rgba(0, 0, 0, 0.1);">
                <h2 style="background-color: #4CAF50; color: #ffffff; padding: 10px; border-radius: 8px 8px 0 0; text-align: center;">Payment Confirmation</h2>
                <div style="padding: 10px;">
                  <p>Dear Customer,</p>
                  <p>We are pleased to inform you that we have successfully received your payment.</p>
                  <p>Here are the details of your transaction:</p>
                  <p><strong>Transaction ID:</strong> T1234567890</p>
                  <p><strong>Amount Paid:</strong> 100.00 USD</p>
                  <p><strong>Payment Date:</strong> January 1, 2023</p>
                  <p><strong>Payment Method:</strong> Credit Card</p>
                  <p>If you have any questions or concerns, please do not hesitate to contact our support team.</p>
                  <p>Thank you for your business!</p>
                  <p>Sincerely,<br/>The Company Team</p>
                </div>
                <div style="text-align: center; padding: 10px; font-size: 12px; color: #888888;">
                  <p>Rusty Templates | Tatu City, Kenya | info@rustytemplates.com</p>
                </div>
              </div>
            </div>
            "#;

            let confirmed_mail = Email {
                recipient: EmailUser {
                    full_name: None,
                    email_address: body.data.customer.email
                },
                subject: "Payment Confirmation".to_string(),
                title: "Payment Received! Thanks!".to_string(),
                body: email_body.to_string()
            };

            if let Err(e) = send_email(headers.clone(), confirmed_mail).await {
                eprintln!("Failed to send email: {:?}", e);
                return (
                    StatusCode::BAD_REQUEST,
                    format!("Transaction successful but could not send email!"),
                ).into_response()
            };
        }
        (StatusCode::CREATED, format!("Transaction successful!")).into_response()
    } else {
        println!("Invalid signature: expected {}, got {}", signature, hash);
        (
            StatusCode::BAD_REQUEST,
            format!("Transaction failed!"),
        ).into_response()
    }
}
