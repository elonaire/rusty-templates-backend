use axum::{
    extract::{Extension, Json},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
};
use hex;
use hmac::{Hmac, Mac};
use hyper::header::COOKIE;
use lib::{
    integration::{
        // email::send_email,
        // file::purchase_product_artifact,
        grpc::clients::{
            acl_service::{acl_client::AclClient, Empty},
            email_service::{
                email_service_client::EmailServiceClient, Email as TonicEmail,
                EmailUser as TonicEmailUser,
            },
            files_service::{files_service_client::FilesServiceClient, PurchaseFileDetails},
        },
        order::{get_all_artifacts_for_order, update_order},
    },
    utils::models::{Email, EmailUser, OrderStatus},
};
use serde_json::Value;
use sha2::Sha512;
use std::{env, sync::Arc};
use surrealdb::{engine::remote::ws::Client, Surreal};

// Type alias for HMAC-SHA512
type HmacSha512 = Hmac<Sha512>;

pub async fn handle_paystack_webhook(
    Extension(_db): Extension<Arc<Surreal<Client>>>,
    headers: HeaderMap,
    Json(body): Json<Value>,
) -> impl IntoResponse {
    // Retrieve the x-paystack-signature header
    let signature = headers
        .get("x-paystack-signature")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    // Get the secret key
    let secret = env::var("PAYSTACK_SECRET").expect("PAYSTACK_SECRET must be set");
    let deployment_env = env::var("ENVIRONMENT").unwrap_or_else(|_| "prod".to_string()); // default to production because it's the most secure

    // Verify the webhook payload
    let mut mac =
        HmacSha512::new_from_slice(secret.as_bytes()).expect("HMAC can take key of any size");
    mac.update(
        serde_json::to_string(&body)
            .expect("Failed to serialize body")
            .as_bytes(),
    );
    let result = mac.finalize();
    let hash = hex::encode(result.into_bytes());

    let paystack_signature_is_valid = match deployment_env.as_str() {
        "prod" => hash == signature,
        _ => true,
    };

    if paystack_signature_is_valid {
        // HMAC validation passed
        if let Some(event) = body.get("event").and_then(|e| e.as_str()) {
            if event == "charge.success" {
                if let Some(data) = body.get("data") {
                    if let Some(reference) = data.get("reference").and_then(|r| r.as_str()) {
                        // Internal sign in logic using gRPC
                        let acl_grpc_client =
                            AclClient::connect("http://[::1]:50051").await.map_err(|e| {
                                tracing::error!("Failed to connect to ACL service: {}", e);
                            });
                        let request = tonic::Request::new(Empty {});

                        if let Ok(auth_res) =
                            acl_grpc_client.unwrap().sign_in_as_service(request).await
                        {
                            let mut header_map = HeaderMap::new();
                            let internal_jwt = auth_res.into_inner().token;
                            header_map.insert(
                                "Authorization",
                                format!("Bearer {:?}", &internal_jwt)
                                    .as_str()
                                    .parse()
                                    .unwrap(),
                            );
                            header_map.insert(
                                COOKIE,
                                format!("oauth_client=;t={:?}", &internal_jwt)
                                    .as_str()
                                    .parse()
                                    .unwrap(),
                            );

                            // Update order status
                            if let Err(e) = update_order(
                                header_map.clone(),
                                reference.to_string(),
                                OrderStatus::Confirmed,
                            )
                            .await
                            {
                                tracing::error!("Failed to update order: {:?}", e);
                                // return (
                                //     StatusCode::BAD_REQUEST,
                                //     format!("Transaction successful but could not update order status!"),
                                // ).into_response();
                            }

                            // give ownership rights to artifacts
                            // TODO: Change to gRPC for this. Implement gRPC server & client for orders service

                            if let Ok(artifacts) = get_all_artifacts_for_order(
                                header_map.clone(),
                                reference.to_string(),
                            )
                            .await
                            {
                                let files_service_grpc_client =
                                    FilesServiceClient::connect("http://[::1]:50053")
                                        .await
                                        .map_err(|e| {
                                            tracing::error!(
                                                "Failed to connect to Files service: {}",
                                                e
                                            );
                                        });

                                for artifact in artifacts.artifacts.iter() {
                                    let request = tonic::Request::new(PurchaseFileDetails {
                                        buyer_id: artifacts.buyer_id.clone(),
                                        file_id: artifact.clone(),
                                    });

                                    // if let Err(e) = purchase_product_artifact(
                                    //     &header_map.clone(),
                                    //     artifact.clone(),
                                    //     artifacts.buyer_id.clone(),
                                    // )
                                    // .await
                                    // {
                                    //     eprintln!("Failed to update artifacts purchases: {:?}, artifact: {}", e, artifact);
                                    // };
                                    if let Err(e) = files_service_grpc_client
                                        .clone()
                                        .unwrap()
                                        .purchase_file(request)
                                        .await
                                    {
                                        tracing::error!("Failed to purchase file: {:?}", e);
                                    }
                                }
                            }

                            // Construct and send confirmation email
                            let confirmed_mail = if let Some(customer) = data.get("customer") {
                                if let Some(email) = customer.get("email").and_then(|e| e.as_str())
                                {
                                    let email_body = format!(
                                        r#"
                                    <div style="font-family: Arial, sans-serif; background-color: #f4f4f4;">
                                        <div style="max-width: 600px; margin: auto; background-color: #ffffff; border-radius: 8px; box-shadow: 0 0 10px rgba(0, 0, 0, 0.1);">
                                            <h2 style="background-color: #4CAF50; color: #ffffff; padding: 10px; border-radius: 8px 8px 0 0; text-align: center;">Payment Confirmation</h2>
                                            <div style="padding: 10px;">
                                                <p>Dear Customer,</p>
                                                <p>We are pleased to inform you that we have successfully received your payment.</p>
                                                <p>Your template is also ready for download. Happy Crabbing 🦀 🚀</p>
                                                <p>
                                                    <a href="https://rustytemplates.com/account" style="display: inline-block; padding: 10px 20px; background-color: #4CAF50; color: white; text-decoration: none; border-radius: 5px;">Download Here</a>
                                                </p>
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
                                        body: email_body.to_string(),
                                    })
                                } else {
                                    None
                                }
                            } else {
                                None
                            };

                            if let Some(email) = confirmed_mail {
                                let email_service_grpc_client =
                                    EmailServiceClient::connect("http://[::1]:50052")
                                        .await
                                        .map_err(|e| {
                                            tracing::error!(
                                                "Failed to connect to Files service: {}",
                                                e
                                            );
                                        });

                                let request = tonic::Request::new(TonicEmail {
                                    recipient: Some(TonicEmailUser {
                                        email_address: email.recipient.email_address,
                                        full_name: match email.recipient.full_name {
                                            Some(full_name) => full_name,
                                            None => "".to_string(),
                                        },
                                    }),
                                    subject: email.subject,
                                    title: email.title,
                                    body: email.body,
                                });

                                if let Err(e) =
                                    email_service_grpc_client.unwrap().send_email(request).await
                                {
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
                (
                    StatusCode::BAD_REQUEST,
                    format!("Unhandled event type: {}", event),
                )
                    .into_response()
            }
        } else {
            (
                StatusCode::BAD_REQUEST,
                format!("Event type missing or invalid"),
            )
                .into_response()
        }
    } else {
        tracing::error!("Invalid signature: expected {}, got {}", signature, hash);
        (StatusCode::BAD_REQUEST, format!("Transaction failed!")).into_response()
    }
}
