use std::sync::Arc;

use crate::graphql::{
    resolvers::cart::mutation::{claim_cart, set_session_cookie},
    schemas::general::{Cart, Order},
};
use async_graphql::{Context, Error, Object, Result};
use axum::{http::HeaderMap, Extension};
use lib::{
    integration::{
        foreign_key::add_foreign_key_if_not_exists, payments::initiate_payment_integration,
        user::get_user_email,
    },
    middleware::auth::graphql::check_auth_from_acl,
    utils::{
        custom_error::ExtendedError,
        models::{ForeignKey, OrderStatus, User, UserPaymentDetails},
    },
};
use surrealdb::{engine::remote::ws::Client, Surreal};

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

            println!("buyer_result: {:?}", buyer_result);

            let _claimed_cart = claim_cart(db, &internal_user_id, &session_id).await?;

            let mut existing_cart_query = db
                .query("SELECT * FROM cart WHERE archived=false AND owner=type::thing($user_id) LIMIT 1")
                .bind(("user_id", format!("user_id:{}", internal_user_id)))
                .await
                .map_err(|e| Error::new(e.to_string()))?;

            let existing_cart: Option<Cart> = existing_cart_query.take(0)?;

            println!("existing_cart: {:?}", existing_cart);

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

                    println!("new_order: {:?}", new_order);

                    let get_user_email_res =
                        get_user_email(ctx, buyer_result.unwrap().user_id.clone()).await;
                    println!("get_user_email_res: {:?}", get_user_email_res);

                    match get_user_email_res {
                        Ok(email) => {
                            let payment_info = UserPaymentDetails {
                                email,
                                amount: cart.total_amount as u64,
                                reference: new_order[0]
                                    .id
                                    .as_ref()
                                    .map(|t| &t.id)
                                    .expect("id")
                                    .to_raw(),
                                // metadata: Some(PaymentDetailsMetaData {
                                //     cart_id: Some(cart_id),
                                // }),
                            };
                            match initiate_payment_integration(ctx, payment_info).await {
                                Ok(payment_link) => Ok(payment_link),
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

            let user_fk = ForeignKey {
                table: "user_id".into(),
                column: "user_id".into(),
                foreign_key: auth_status.sub.clone(),
            };

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

            let mut existing_order_query = db
                .query("SELECT * FROM order WHERE id=type::thing($id) AND in=type::thing($user_id) LIMIT 1")
                .bind(("user_id", format!("user_id:{}", internal_user_id)))
                .bind(("id", format!("order:{}", order_id)))
                .await
                .map_err(|e| Error::new(e.to_string()))?;

            let existing_order: Option<Order> = existing_order_query.take(0)?;

            match existing_order {
                Some(order) => {
                    let mut update_order_transaction = db
                        .query(
                            "
                        BEGIN TRANSACTION;
                        LET $order = type::thing($order_id);
                        LET $new_order = UPDATE ONLY $order SET status = $new_status;
                        RETURN $new_order;
                        COMMIT TRANSACTION;
                        ",
                        )
                        .bind((
                            "order_id",
                            format!(
                                "order:{}",
                                order.id.as_ref().map(|t| &t.id).expect("id").to_raw()
                            ),
                        ))
                        .bind(("new_status", status))
                        .await
                        .map_err(|e| Error::new(e.to_string()))?;

                    let response: Option<Order> = update_order_transaction.take(0)?;

                    match status {
                        OrderStatus::Confirmed => {
                            let mut _update_order_transaction = db
                            .query(
                                "
                                LET $order = type::thing($order_id);
                                LET $active_cart = (SELECT VALUE ->(cart WHERE archived=false) FROM ONLY $order LIMIT 1)[0];
                                LET $updated = (UPDATE ONLY $active_cart SET archived=true);

                                RETURN $updated;
                                "
                            )
                            .bind(("order_id", format!("order:{}", order.id.as_ref().map(|t| &t.id).expect("id").to_raw())))
                            .await
                            .map_err(|e| Error::new(e.to_string()))?;
                        }
                        _ => {}
                    }

                    match response {
                        Some(updated_order) => Ok(format!("{:?}", updated_order.status)),
                        None => {
                            Err(ExtendedError::new("Cart is empty!", Some(400.to_string())).build())
                        }
                    }

                    // Ok(response)
                }
                None => Err(ExtendedError::new("Cart is empty!", Some(400.to_string())).build()),
            }
        } else {
            Err(ExtendedError::new("Cart is empty!", Some(400.to_string())).build())
        }
    }
}
