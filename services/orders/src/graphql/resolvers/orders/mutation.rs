use std::sync::Arc;

use crate::{
    graphql::{
        resolvers::cart::mutation::{claim_cart, set_session_cookie},
        schemas::general::{Cart, Order},
    },
    utils::orders::update_order,
};
use async_graphql::{Context, Error, Object, Result};
use axum::{http::HeaderMap, Extension};
use hyper::header::{AUTHORIZATION, COOKIE};
use lib::{
    integration::{
        foreign_key::add_foreign_key_if_not_exists,
        grpc::clients::{
            acl_service::{acl_client::AclClient, GetUserEmailRequest},
            payments_service::{
                payments_service_client::PaymentsServiceClient, UserPaymentDetails,
            },
        },
        // payments::initiate_payment_integration,
    },
    middleware::auth::graphql::check_auth_from_acl,
    utils::{
        custom_error::ExtendedError,
        grpc::{create_grpc_client, AuthMetaData},
        models::{ForeignKey, OrderStatus, User},
    },
};
use surrealdb::{engine::remote::ws::Client, Surreal};
use tonic::transport::Channel;

#[derive(Default)]
pub struct OrderMutation;

#[Object]
impl OrderMutation {
    pub async fn create_order(&self, ctx: &Context<'_>) -> Result<String> {
        let db = ctx.data::<Extension<Arc<Surreal<Client>>>>().unwrap();

        if let Some(headers) = ctx.data_opt::<HeaderMap>() {
            let auth_status = check_auth_from_acl(headers).await?;

            let user_fk = ForeignKey {
                table: "user_id".into(),
                column: "user_id".into(),
                foreign_key: auth_status.sub.clone(),
            };

            let session_id = set_session_cookie(&mut headers.clone(), ctx);

            let buyer_result =
                add_foreign_key_if_not_exists::<Extension<Arc<Surreal<Client>>>, User>(db, user_fk)
                    .await;
            let buyer_result_clone = buyer_result.clone();
            let internal_user_id = buyer_result_clone
                .unwrap()
                .id
                .as_ref()
                .map(|t| &t.id)
                .expect("id")
                .to_raw();

            let _claimed_cart = claim_cart(db, &internal_user_id, &session_id).await?;

            let mut existing_cart_query = db
                .query("SELECT * FROM cart WHERE archived=false AND owner=type::thing($user_id) LIMIT 1")
                .bind(("user_id", format!("user_id:{}", internal_user_id)))
                .await
                .map_err(|e| Error::new(e.to_string()))?;

            let existing_cart: Option<Cart> = existing_cart_query.take(0)?;

            match existing_cart {
                Some(cart) => {
                    let mut create_order_transaction = db
                        .query(
                            "
                        BEGIN TRANSACTION;
                        LET $user = type::thing($user_id);
                        LET $cart = type::thing($cart_id);
                        LET $new_order = (RELATE $user -> order -> $cart CONTENT {
                            status: 'Pending',
                        } RETURN AFTER);
                        RETURN $new_order;
                        COMMIT TRANSACTION;
                        ",
                        )
                        // .bind(("comment_body", comment))
                        .bind(("user_id", format!("user_id:{}", internal_user_id)))
                        .bind((
                            "cart_id",
                            format!(
                                "cart:{}",
                                cart.id.as_ref().map(|t| &t.id).expect("id").to_raw()
                            ),
                        ))
                        .await
                        .map_err(|e| Error::new(e.to_string()))?;

                    let new_order: Vec<Order> = create_order_transaction.take(0)?;

                    let auth_header = headers.get(AUTHORIZATION);
                    let cookie_header = headers.get(COOKIE);

                    let mut request = tonic::Request::new(GetUserEmailRequest {
                        user_id: buyer_result.unwrap().user_id.clone(),
                    });

                    let auth_metadata: AuthMetaData<GetUserEmailRequest> = AuthMetaData {
                        auth_header,
                        cookie_header,
                        constructed_grpc_request: Some(&mut request),
                    };

                    let mut acl_grpc_client = create_grpc_client::<
                        GetUserEmailRequest,
                        AclClient<Channel>,
                    >(
                        "http://[::1]:50051", true, Some(auth_metadata)
                    )
                    .await
                    .map_err(|e| {
                        tracing::error!("Failed to connect to ACL service: {}", e);
                        ExtendedError::new(
                            "Failed to connect to ACL service",
                            Some(500.to_string()),
                        )
                        .build()
                    })?;

                    let get_user_email_res = acl_grpc_client.get_user_email(request).await;

                    match get_user_email_res {
                        Ok(email) => {
                            let payment_info = UserPaymentDetails {
                                email: email.into_inner().email,
                                amount: cart.total_amount as u64,
                                reference: new_order[0]
                                    .id
                                    .as_ref()
                                    .map(|t| &t.id)
                                    .expect("id")
                                    .to_raw(),
                            };

                            let mut request = tonic::Request::new(payment_info);

                            let auth_metadata: AuthMetaData<UserPaymentDetails> = AuthMetaData {
                                auth_header,
                                cookie_header,
                                constructed_grpc_request: Some(&mut request),
                            };

                            let mut payments_grpc_client =
                                create_grpc_client::<
                                    UserPaymentDetails,
                                    PaymentsServiceClient<Channel>,
                                >(
                                    "http://[::1]:50056", true, Some(auth_metadata)
                                )
                                .await
                                .map_err(|e| {
                                    tracing::error!("Failed to connect to Payments service: {}", e);
                                    ExtendedError::new(
                                        "Failed to connect to Payments service",
                                        Some(400.to_string()),
                                    )
                                    .build()
                                })?;

                            match payments_grpc_client
                                .initiate_payment_integration(request)
                                .await
                            {
                                Ok(payment_link) => Ok(payment_link.into_inner().authorization_url),
                                Err(e) => Err(ExtendedError::new(
                                    format!("Error getting payment link! {:?}", e),
                                    Some(400.to_string()),
                                )
                                .build()),
                            }
                        }
                        Err(e) => Err(ExtendedError::new(
                            format!("User not found! {:?}", e),
                            Some(400.to_string()),
                        )
                        .build()),
                    }

                    // Ok(response)
                }
                None => Err(ExtendedError::new("Cart is empty!", Some(400.to_string())).build()),
            }
        } else {
            Err(ExtendedError::new("Invalid Request!", Some(400.to_string())).build())
        }
    }

    pub async fn update_order(
        &self,
        ctx: &Context<'_>,
        order_id: String,
        status: OrderStatus,
    ) -> Result<String> {
        let db = ctx.data::<Extension<Arc<Surreal<Client>>>>().unwrap();

        if let Some(headers) = ctx.data_opt::<HeaderMap>() {
            let auth_status = check_auth_from_acl(headers).await?;

            let updated_order =
                update_order(db, auth_status.sub.as_str(), order_id.as_str(), status).await?;
            Ok(updated_order)
        } else {
            Err(ExtendedError::new("Cart is empty!", Some(400.to_string())).build())
        }
    }
}
