use std::sync::Arc;

use crate::graphql::schemas::general::{Cart, CartOperation};
use async_graphql::{Context, Object, Result};
use axum::{http::HeaderMap, Extension};
use hyper::{header::SET_COOKIE, StatusCode};
use lib::{
    integration::foreign_key::add_foreign_key_if_not_exists,
    middleware::auth::graphql::check_auth_from_acl,
    utils::{
        custom_error::ExtendedError,
        models::{ForeignKey, ProductSku, User},
    },
};
use surrealdb::{engine::remote::ws::Client, Surreal};
use uuid::Uuid;

struct UpdateCartArgs {
    pub cart: Cart,
    pub internal_product_sku_id: String,
    pub cart_operation: CartOperation,
    pub db_ctx: Extension<Arc<Surreal<Client>>>,
}

#[derive(Debug)]
struct NewCartArgs {
    pub internal_product_sku_id: String,
    pub internal_user_id: Option<String>,
    pub db_ctx: Extension<Arc<Surreal<Client>>>,
    pub session_id: String,
}

#[derive(Default)]
pub struct CartMutation;

#[Object]
impl CartMutation {
    /// Resolver method to create/update an instance of a cart
    pub async fn create_or_update_cart(
        &self,
        ctx: &Context<'_>,
        external_product_sku_id: String,
        cart_operation: CartOperation,
    ) -> Result<Cart> {
        let db = ctx.data::<Extension<Arc<Surreal<Client>>>>().unwrap();

        if let Some(headers) = ctx.data_opt::<HeaderMap>() {
            let session_id = set_session_cookie(&mut headers.clone(), ctx);

            let product_sku_fk_body = ForeignKey {
                table: "product_sku_id".into(),
                column: "product_sku_id".into(),
                foreign_key: external_product_sku_id.clone(),
            };

            let product_sku_fk = add_foreign_key_if_not_exists::<
                Extension<Arc<Surreal<Client>>>,
                ProductSku,
            >(db, product_sku_fk_body)
            .await;

            let internal_product_sku_id = product_sku_fk
                .unwrap()
                .id
                .as_ref()
                .map(|t| &t.id)
                .expect("id")
                .to_raw();

            // let auth_header = headers.get(AUTHORIZATION);
            // let cookie_header = headers.get(COOKIE);

            match check_auth_from_acl(headers).await {
                Ok(auth_status) => {
                    let user_fk_body = ForeignKey {
                        table: "user_id".into(),
                        column: "user_id".into(),
                        foreign_key: auth_status.sub,
                    };

                    let user_fk = add_foreign_key_if_not_exists::<
                        Extension<Arc<Surreal<Client>>>,
                        User,
                    >(db, user_fk_body)
                    .await;

                    let internal_user_id = user_fk
                        .unwrap()
                        .id
                        .as_ref()
                        .map(|t| &t.id)
                        .expect("id")
                        .to_raw();

                    let _claimed_cart = claim_cart(db, &internal_user_id, &session_id).await;

                    let mut existing_cart_query = db
                        .query("SELECT *, (SELECT <-product_sku_id.product_sku_id[0] AS product_sku_id, quantity FROM <-cart_product[*]) AS products FROM ONLY cart WHERE archived=false AND owner=type::thing($user_id) LIMIT 1")
                        .bind(("user_id", format!("user_id:{}", internal_user_id)))
                        .await
                        .map_err(|e| {
                            tracing::error!("Error retrieving existing cart: {:?}", e);
                            ExtendedError::new("Failed to add to cart!", Some(StatusCode::BAD_REQUEST.as_u16()))
                                .build()
                        })?;

                    let existing_cart: Option<Cart> = existing_cart_query.take(0).map_err(|e| {
                        tracing::error!("(Deserialization)Error retrieving existing cart: {:?}", e);
                        ExtendedError::new(
                            "Failed to add to cart!",
                            Some(StatusCode::BAD_REQUEST.as_u16()),
                        )
                        .build()
                    })?;

                    match existing_cart {
                        Some(cart) => {
                            let update_args = UpdateCartArgs {
                                cart: cart.clone(),
                                internal_product_sku_id,
                                cart_operation,
                                db_ctx: db.clone(),
                            };

                            let updated_cart = update_existing_cart(update_args).await;
                            tracing::debug!(
                                "updated_cart(auth true, existing cart): {:?}",
                                updated_cart
                            );

                            updated_cart
                        }
                        None => {
                            let new_cart_args = NewCartArgs {
                                internal_product_sku_id,
                                internal_user_id: Some(internal_user_id),
                                db_ctx: db.clone(),
                                session_id: session_id.clone(),
                            };

                            let new_cart = create_new_cart(new_cart_args).await.map_err(|e| {
                                tracing::error!("Error creating new cart: {:?}", e);
                                ExtendedError::new(
                                    "Failed to add to cart!",
                                    Some(StatusCode::BAD_REQUEST.as_u16()),
                                )
                                .build()
                            });

                            new_cart
                        }
                    }
                }
                Err(e) => {
                    tracing::debug!("Failed new cart, trying existing: {:?}", e);
                    let mut existing_cart_query = db
                        .query("SELECT *, (SELECT <-product_sku_id.product_sku_id[0] AS product_sku_id, quantity FROM <-cart_product[*]) AS products FROM ONLY cart WHERE archived=false AND session_id=$session_id LIMIT 1")
                        .bind(("session_id", session_id.clone()))
                        .await
                        .map_err(|e| {
                            tracing::error!("DB Query Error: {}", e);
                            ExtendedError::new("Failed to add to cart!", Some(StatusCode::BAD_REQUEST.as_u16())).build()
                        })?;

                    let existing_cart: Option<Cart> = existing_cart_query.take(0).map_err(|e| {
                        tracing::error!("Deserialization Error: {}", e);
                        ExtendedError::new(
                            "Failed to add to cart!",
                            Some(StatusCode::BAD_REQUEST.as_u16()),
                        )
                        .build()
                    })?;

                    match existing_cart {
                        Some(cart) => {
                            let update_args = UpdateCartArgs {
                                cart: cart.clone(),
                                internal_product_sku_id,
                                cart_operation,
                                db_ctx: db.clone(),
                            };

                            let updated_cart = update_existing_cart(update_args).await;
                            tracing::debug!(
                                "updated_cart(auth false, existing cart): {:?}",
                                updated_cart
                            );

                            updated_cart
                        }
                        None => {
                            let new_cart_args = NewCartArgs {
                                internal_product_sku_id,
                                internal_user_id: None,
                                db_ctx: db.clone(),
                                session_id: session_id.clone(),
                            };

                            let new_cart = create_new_cart(new_cart_args).await;
                            tracing::debug!(
                                "new_cart(auth false, no existing cart): {:?}",
                                new_cart
                            );

                            new_cart
                        }
                    }
                }
            }
        } else {
            Err(
                ExtendedError::new("Invalid Request!", Some(StatusCode::BAD_REQUEST.as_u16()))
                    .build(),
            )
        }
    }
}

/// Utility function to update an instance of a cart
async fn update_existing_cart(args: UpdateCartArgs) -> Result<Cart> {
    let cart_id_raw = args.cart.id.as_ref().map(|t| &t.id).expect("id").to_raw();
    tracing::debug!("Updating existing cart");

    match args.cart_operation {
        CartOperation::AddProduct => {
            let mut update_cart_transaction = args
                .db_ctx
                .query(
                    "
                BEGIN TRANSACTION;
                LET $product_sku_id = type::thing('product_sku_id', $internal_product_sku_id);
                LET $cart = type::thing('cart', $cart_id);

                RELATE $product_sku_id -> cart_product -> $cart CONTENT {
                    quantity: 1,
                };

                LET $updated_cart = SELECT *, (SELECT <-product_sku_id.product_sku_id[0] AS product_sku_id, quantity FROM <-cart_product[*]) AS products FROM ONLY cart WHERE archived=false AND session_id=$session_id LIMIT 1;

                RETURN $updated_cart;
                COMMIT TRANSACTION;
                ",
                )
                .bind(("internal_product_sku_id", args.internal_product_sku_id))
                .bind(("cart_id", cart_id_raw))
                .await
                .map_err(|e| {
                    tracing::error!("DB Query Error: {}", e);
                    ExtendedError::new("Failed to add to cart!", Some(StatusCode::BAD_REQUEST.as_u16())).build()
                })?;

            let response: Vec<Cart> = update_cart_transaction.take(0).map_err(|e| {
                tracing::error!("Deserialization Error: {}", e);
                ExtendedError::new(
                    "Failed to add to cart!",
                    Some(StatusCode::BAD_REQUEST.as_u16()),
                )
                .build()
            })?;
            Ok(response.first().unwrap().to_owned())
        }
        CartOperation::RemoveProduct => {
            let mut update_cart_transaction = args
                .db_ctx
                .query(
                    "
                BEGIN TRANSACTION;
                LET $product_sku_id = type::thing('product_sku_id', $internal_product_sku_id);
                LET $cart = type::thing('cart', $cart_id);

                DELETE cart_product->$cart WHERE <-(product_sku_id WHERE id = $product_sku_id);

                LET $updated_cart = SELECT *, (SELECT <-product_sku_id.product_sku_id[0] AS product_sku_id, quantity FROM <-cart_product[*]) AS products FROM ONLY $cart;

                RETURN $updated_cart;
                COMMIT TRANSACTION;
                ",
                )
                .bind(("internal_product_sku_id", args.internal_product_sku_id))
                .bind(("cart_id", cart_id_raw))
                .await
                .map_err(|e| {
                    tracing::error!("DB Query Error: {}", e);
                    ExtendedError::new("Failed to add to cart!", Some(StatusCode::BAD_REQUEST.as_u16())).build()
                })?;

            let response: Vec<Cart> = update_cart_transaction.take(0).map_err(|e| {
                tracing::error!("Deserialization Error: {}", e);
                ExtendedError::new(
                    "Failed to add to cart!",
                    Some(StatusCode::BAD_REQUEST.as_u16()),
                )
                .build()
            })?;
            Ok(response.first().unwrap().to_owned())
        }
    }
}

/// Utility function to create an instance of a cart
async fn create_new_cart(args: NewCartArgs) -> Result<Cart> {
    tracing::debug!("args: {:?}", args);
    let mut create_cart_transaction = args
        .db_ctx
        .query(
            "
            BEGIN TRANSACTION;
            LET $product_sku_id = type::thing('product_sku_id', $internal_product_sku_id);
            LET $owner = IF $user = NONE
            { NONE }
                ELSE
            { type::thing('user_id', $user) }
            ;
            CREATE cart CONTENT {
               	owner: $owner,
               	session_id: $session_id,
            };
            LET $cart_id = (SELECT VALUE id FROM ONLY $new_cart);
            RELATE $product_sku_id -> cart_product -> $cart_id CONTENT {
                quantity: 1
            };

            LET $new_cart = SELECT *, (SELECT <-product_sku_id.product_sku_id[0] AS product_sku_id, quantity FROM <-cart_product[*]) AS products FROM ONLY $cart_id;
            RETURN $new_cart;
            COMMIT TRANSACTION;
        ",
        )
        .bind(("internal_product_sku_id", args.internal_product_sku_id))
        .bind(("user", args.internal_user_id))
        .bind(("session_id", args.session_id))
        .await
        .map_err(|e| {
            tracing::error!("(create_cart_transaction)DB Query Error: {}", e);
            ExtendedError::new("Failed to add to cart!", Some(StatusCode::BAD_REQUEST.as_u16())).build()
        })?;

    let response: Vec<Cart> = create_cart_transaction.take(0).map_err(|e| {
        tracing::error!("(create_cart_transaction)Deserialization Error: {}", e);
        ExtendedError::new(
            "Failed to add to cart!",
            Some(StatusCode::BAD_REQUEST.as_u16()),
        )
        .build()
    })?;

    Ok(response.first().unwrap().to_owned())
}

/// Utility function to set a session cookie for the cart
pub fn set_session_cookie(headers: &mut HeaderMap, ctx: &Context<'_>) -> String {
    // Handle anonymous users
    if let Some(session_cookie) = headers
        .get("Cookie")
        .and_then(|c| c.to_str().ok())
        .and_then(|c| c.split("; ").find(|&s| s.starts_with("session_id=")))
    {
        tracing::debug!("Cookie: {}", session_cookie);
        session_cookie.trim_start_matches("session_id=").to_string()
    } else {
        // Generate a new session ID for anonymous user
        let session_id = Uuid::new_v4().to_string();
        // Send back the new session cookie header to the client
        // This works if you have a mechanism to return headers in response
        ctx.insert_http_header(
            SET_COOKIE,
            format!("session_id={}; Path=/; HttpOnly", session_id),
        );

        session_id
    }
}

/// Utility function to claim a cart that was instantiated anonymously
pub async fn claim_cart(
    db: &Extension<Arc<Surreal<Client>>>,
    internal_user_id: &String,
    session_id: &String,
) -> Result<Option<Cart>> {
    let mut existing_cart_query = db
        .query(
            "
            LET $owner = type::thing($internal_user_id);
            LET $updates = UPDATE cart SET owner = $owner WHERE session_id = $session_id RETURN AFTER;
            RETURN $updates[0];
            "
        )
        .bind(("session_id", session_id.clone()))
        .bind(("internal_user_id", format!("user_id:{}", &internal_user_id)))
        .await
        .map_err(|e| {
            tracing::error!("DB Query Error: {}", e);
            ExtendedError::new("Failed to claim cart!", Some(StatusCode::BAD_REQUEST.as_u16())).build()
        })?;

    let existing_cart: Option<Cart> = existing_cart_query.take(0).map_err(|e| {
        tracing::error!("Deserialization Error: {}", e);
        ExtendedError::new(
            "Failed to claim cart!",
            Some(StatusCode::BAD_REQUEST.as_u16()),
        )
        .build()
    })?;

    Ok(existing_cart)
}
