use axum::{
    extract::{Extension, Json},
    http::{HeaderMap, HeaderValue, StatusCode},
    response::IntoResponse,
};
use hex;
use hmac::{Hmac, Mac};
use hyper::header::{AUTHORIZATION, COOKIE};
use lib::{
    integration::grpc::clients::{
        acl_service::{acl_client::AclClient, Empty},
        email_service::{
            email_service_client::EmailServiceClient, Email as TonicEmail,
            EmailUser as TonicEmailUser,
        },
        files_service::{files_service_client::FilesServiceClient, PurchaseFileDetails},
        orders_service::{
            orders_service_client::OrdersServiceClient, GetAllArtifactsForOrderPayload,
        },
    },
    utils::{
        grpc::{create_grpc_client, AuthMetaData},
        models::{Email, EmailUser},
    },
};
use rumqttc::v5::mqttbytes::QoS;
use serde_json::Value;
use sha2::Sha512;
use std::{env, sync::Arc};
use surrealdb::{engine::remote::ws::Client, Surreal};
use tonic::transport::Channel;

use crate::AppState;

// Type alias for HMAC-SHA512
type HmacSha512 = Hmac<Sha512>;

pub async fn handle_paystack_webhook(
    Extension(_db): Extension<Arc<Surreal<Client>>>,
    Extension(shared_state): Extension<Arc<AppState>>,
    headers: HeaderMap,
    Json(body): Json<Value>,
) -> impl IntoResponse {
    // Retrieve the x-paystack-signature header
    let signature = headers
        .get("x-paystack-signature")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    // Get the secret key
    let secret = env::var("PAYSTACK_SECRET");

    if let Err(e) = secret {
        tracing::error!("Missing the PAYSTACK_SECRET environment variable.: {}", e);
        return (StatusCode::INTERNAL_SERVER_ERROR, "Server Error").into_response();
    }
    let secret = secret.unwrap();

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
                        let owned_reference = reference.to_string();

                        if let Err(e) = shared_state
                            .mqtt_client
                            .publish(
                                "payment/successful",
                                QoS::ExactlyOnce,
                                false,
                                owned_reference,
                            )
                            .await
                        {
                            tracing::error!("Failed to publish payment successful event: {}", e);
                        }

                        // tracing::debug!("client: {:?}", client);
                        // Internal sign in logic using gRPC
                        let request = tonic::Request::new(Empty {});

                        let acl_service_grpc = env::var("OAUTH_SERVICE_GRPC");

                        if let Err(e) = acl_service_grpc {
                            tracing::error!(
                                "Failed to get OAUTH_SERVICE_GRPC environment variable: {}",
                                e
                            );
                            return (StatusCode::INTERNAL_SERVER_ERROR, "Server Error.")
                                .into_response();
                        }

                        let acl_service_grpc = acl_service_grpc.unwrap();

                        if let Ok(mut acl_grpc_client) = create_grpc_client::<
                            Empty,
                            AclClient<Channel>,
                        >(
                            &acl_service_grpc, false, None
                        )
                        .await
                        .map_err(|e| {
                            tracing::error!("Failed to connect to ACL service: {}", e);
                            return (
                                StatusCode::SERVICE_UNAVAILABLE,
                                "Transaction successful but failed to authenticate request.",
                            )
                                .into_response();
                        }) {
                            if let Ok(auth_res) = acl_grpc_client.sign_in_as_service(request).await
                            {
                                let mut header_map = HeaderMap::new();
                                let internal_jwt = auth_res.into_inner().token;

                                if internal_jwt.as_str().parse::<HeaderValue>().is_err() {
                                    tracing::error!("Failed to parse str to HeaderValue");
                                    return (
                                                StatusCode::NOT_FOUND,
                                                format!(
                                                "Transaction successful but could not reach Orders service!"
                                            ),
                                            )
                                                .into_response();
                                }

                                header_map.insert(
                                    AUTHORIZATION,
                                    format!("Bearer {}", &internal_jwt)
                                        .as_str()
                                        .parse()
                                        .unwrap(),
                                );
                                header_map.insert(
                                    COOKIE,
                                    format!("oauth_client=;t={}", &internal_jwt)
                                        .as_str()
                                        .parse()
                                        .unwrap(),
                                );

                                let auth_header = header_map.get(AUTHORIZATION);
                                let cookie_header = header_map.get(COOKIE);

                                tracing::debug!("auth_header: {:?}", auth_header);

                                let mut request =
                                    tonic::Request::new(GetAllArtifactsForOrderPayload {
                                        order_id: reference.to_string(),
                                    });

                                let auth_metadata: AuthMetaData<GetAllArtifactsForOrderPayload> =
                                    AuthMetaData {
                                        auth_header,
                                        cookie_header,
                                        constructed_grpc_request: Some(&mut request),
                                    };

                                let orders_service_grpc = env::var("ORDERS_SERVICE_GRPC");

                                if let Err(e) = orders_service_grpc {
                                    tracing::error!("Failed to get ORDERS_SERVICE_GRPC environment variable: {}", e);
                                    return (StatusCode::INTERNAL_SERVER_ERROR, "Server Error.")
                                        .into_response();
                                }

                                let orders_service_grpc = orders_service_grpc.unwrap();

                                // give ownership rights to artifacts
                                if let Ok(mut orders_grpc_client) = create_grpc_client::<
                                    GetAllArtifactsForOrderPayload,
                                    OrdersServiceClient<Channel>,
                                >(
                                    &orders_service_grpc, true, Some(auth_metadata)
                                )
                                .await
                                .map_err(|e| {
                                    tracing::error!(
                                        "Failed to connect to Orders service: {}",
                                        e
                                    );
                                     (
                                            StatusCode::NOT_FOUND,
                                            format!(
                                            "Transaction successful but could not reach Orders service!"
                                        ),
                                        )
                                            .into_response()
                                })
                                {
                                    if let Ok(artifacts) = orders_grpc_client
                                        .get_all_artifacts_for_order(request)
                                        .await
                                    {
                                        let artifacts = artifacts.into_inner();

                                        for artifact in artifacts.artifacts.iter() {
                                            tracing::debug!("Found buyer_id: {:?}", artifacts.buyer_id);
                                            let mut request =
                                                tonic::Request::new(PurchaseFileDetails {
                                                    buyer_id: artifacts.buyer_id.clone(),
                                                    file_id: artifact.clone(),
                                                });

                                            let auth_metadata: AuthMetaData<PurchaseFileDetails> =
                                                AuthMetaData {
                                                    auth_header,
                                                    cookie_header,
                                                    constructed_grpc_request: Some(&mut request),
                                                };

                                            let files_service_grpc = env::var("FILES_SERVICE_GRPC");

                                            if let Err(e) = files_service_grpc {
                                                tracing::error!("Failed to get FILES_SERVICE_GRPC environment variable: {}", e);
                                                return (
                                                       StatusCode::INTERNAL_SERVER_ERROR,
                                                       "Server Error",
                                                   )
                                                       .into_response();
                                            }

                                            if let Ok(mut files_service_grpc_client) = create_grpc_client::<
                                                PurchaseFileDetails,
                                                FilesServiceClient<Channel>,
                                            >(
                                                &files_service_grpc.unwrap(), true, Some(auth_metadata)
                                            )
                                            .await
                                            .map_err(|e| {
                                                tracing::error!("Transaction successful but could not reach Files service: {}", e);
                                                (
                                                    StatusCode::NOT_FOUND,
                                                    format!("Transaction successful but could not reach Files service."),
                                                )
                                                    .into_response()
                                            }) {
                                                if let Err(e) =
                                                    files_service_grpc_client.purchase_file(request).await
                                                {
                                                    tracing::error!("Failed to purchase file: {:?}", e);
                                                }
                                            }

                                            tracing::debug!("rest webhook: purchased artifacts!");
                                        }
                                    }
                                }

                                // Construct and send confirmation email
                                let confirmed_mail = if let Some(customer) = data.get("customer") {
                                    if let Some(email) =
                                        customer.get("email").and_then(|e| e.as_str())
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
                                    let mut request = tonic::Request::new(TonicEmail {
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

                                    // let auth_header = headers.get(AUTHORIZATION);
                                    // let cookie_header = headers.get(COOKIE);

                                    let auth_metadata: AuthMetaData<TonicEmail> = AuthMetaData {
                                        auth_header,
                                        cookie_header,
                                        constructed_grpc_request: Some(&mut request),
                                    };

                                    let email_service_grpc = env::var("EMAIL_SERVICE_GRPC");

                                    if let Err(e) = email_service_grpc {
                                        tracing::error!("Failed to get EMAIL_SERVICE_GRPC: {}", e);
                                        return (StatusCode::INTERNAL_SERVER_ERROR, "Server Error")
                                            .into_response();
                                    }

                                    if let Ok(mut email_service_grpc_client) =
                                        create_grpc_client::<TonicEmail, EmailServiceClient<Channel>>(
                                            &email_service_grpc.unwrap(),
                                            true,
                                            Some(auth_metadata),
                                        )
                                        .await
                                        .map_err(|e| {
                                            tracing::error!("Failed to connect to Files service: {}", e);
                                            (
                                                    StatusCode::NOT_FOUND,
                                                    format!("Transaction successful but could not reach Email service!"),
                                                )
                                                    .into_response()
                                        }) {
                                            if let Err(e) =
                                                email_service_grpc_client.send_email(request).await
                                            {
                                                eprintln!("Failed to send email: {:?}", e);
                                                return (
                                                    StatusCode::BAD_REQUEST,
                                                    format!(
                                                        "Transaction successful but could not send email!"
                                                    ),
                                                )
                                                    .into_response();
                                            }
                                        };

                                    tracing::debug!("rest webhook: sent email!");
                                }
                            };
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
