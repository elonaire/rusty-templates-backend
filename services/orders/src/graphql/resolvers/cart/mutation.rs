use std::sync::Arc;

use crate::graphql::schemas::general::{Cart, CartOperation};
use async_graphql::{Context, Error, Object, Result};
use axum::{http::HeaderMap, Extension};
use hyper::header::{AUTHORIZATION, COOKIE, SET_COOKIE};
use lib::{
    integration::{
        foreign_key::add_foreign_key_if_not_exists,
        grpc::clients::products_service::{
            products_service_client::ProductsServiceClient, GetLicensePriceFactorArgs, ProductId,
            RetrieveProductArtifactArgs,
        },
    },
    middleware::auth::graphql::check_auth_from_acl,
    utils::{
        custom_error::ExtendedError,
        grpc::{create_grpc_client, AuthMetaData},
        models::{ForeignKey, License, Product, User},
    },
};
use surrealdb::{engine::remote::ws::Client, Surreal};
use tonic::transport::Channel;
use uuid::Uuid;

struct UpdateCartArgs {
    pub cart: Cart,
    pub internal_product_id: String,
    pub cart_operation: CartOperation,
    pub product_price: u64,
    pub db_ctx: Extension<Arc<Surreal<Client>>>,
    pub license_id: String,
    pub artifact: String,
}

#[derive(Debug)]
struct NewCartArgs {
    pub internal_product_id: String,
    pub product_price: u64,
    pub internal_user_id: Option<String>,
    pub db_ctx: Extension<Arc<Surreal<Client>>>,
    pub session_id: String,
    pub license_id: String,
    pub artifact: String,
    pub license_price_factor: u64,
}

#[derive(Default)]
pub struct CartMutation;

#[Object]
impl CartMutation {
    /// Resolver method to create/update an instance of a cart
    pub async fn create_or_update_cart(
        &self,
        ctx: &Context<'_>,
        external_product_id: String,
        cart_operation: CartOperation,
        external_license_id: String,
    ) -> Result<Cart> {
        let db = ctx.data::<Extension<Arc<Surreal<Client>>>>().unwrap();

        if let Some(headers) = ctx.data_opt::<HeaderMap>() {
            let session_id = set_session_cookie(&mut headers.clone(), ctx);

            let product_fk_body = ForeignKey {
                table: "product_id".into(),
                column: "product_id".into(),
                foreign_key: external_product_id.clone(),
            };

            let license_fk_body = ForeignKey {
                table: "license_id".into(),
                column: "license_id".into(),
                foreign_key: external_license_id.clone(),
            };

            let product_fk = add_foreign_key_if_not_exists::<
                Extension<Arc<Surreal<Client>>>,
                Product,
            >(db, product_fk_body)
            .await;
            let license_fk = add_foreign_key_if_not_exists::<
                Extension<Arc<Surreal<Client>>>,
                License,
            >(db, license_fk_body)
            .await;

            let internal_product_id = product_fk
                .unwrap()
                .id
                .as_ref()
                .map(|t| &t.id)
                .expect("id")
                .to_raw();
            let internal_license_id = license_fk
                .as_ref()
                .unwrap()
                .id
                .as_ref()
                .map(|t| &t.id)
                .expect("id")
                .to_raw();

            let auth_header = headers.get(AUTHORIZATION);
            let cookie_header = headers.get(COOKIE);

            let mut get_product_price_request = tonic::Request::new(ProductId {
                product_id: external_product_id.clone(),
            });

            let auth_metadata: AuthMetaData<ProductId> = AuthMetaData {
                auth_header,
                cookie_header,
                constructed_grpc_request: Some(&mut get_product_price_request),
            };

            let mut products_grpc_client = create_grpc_client::<
                ProductId,
                ProductsServiceClient<Channel>,
            >(
                "http://[::1]:50054", true, Some(auth_metadata)
            )
            .await
            .map_err(|e| {
                tracing::error!("Failed to connect to Products service: {}", e);
                ExtendedError::new(
                    "Failed to connect to Products service",
                    Some(400.to_string()),
                )
                .build()
            })?;

            let product_price = products_grpc_client
                .get_product_price(get_product_price_request)
                .await?
                .into_inner()
                .price;

            let get_product_artifact_request = tonic::Request::new(RetrieveProductArtifactArgs {
                product_id: external_product_id.clone(),
                license_id: external_license_id.clone(),
            });
            let product_artifact = products_grpc_client
                .get_product_artifact(get_product_artifact_request)
                .await?
                .into_inner()
                .artifact;

            tracing::debug!("product_artifact: {:?}", product_artifact);

            let get_license_price_factor_request = tonic::Request::new(GetLicensePriceFactorArgs {
                license_id: external_license_id.clone(),
            });
            let license_price_factor = products_grpc_client
                .get_license_price_factor(get_license_price_factor_request)
                .await?
                .into_inner()
                .price_factor;

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
                        .query("SELECT * FROM cart WHERE archived=false AND owner=type::thing($user_id) LIMIT 1")
                        .bind(("user_id", format!("user_id:{}", internal_user_id)))
                        .await
                        .map_err(|e| Error::new(e.to_string()))?;

                    let existing_cart: Option<Cart> = existing_cart_query.take(0)?;

                    match existing_cart {
                        Some(cart) => {
                            let update_args = UpdateCartArgs {
                                cart: cart.clone(),
                                internal_product_id,
                                cart_operation,
                                product_price,
                                db_ctx: db.clone(),
                                license_id: internal_license_id,
                                artifact: product_artifact,
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
                                internal_product_id,
                                product_price,
                                internal_user_id: Some(internal_user_id),
                                db_ctx: db.clone(),
                                session_id: session_id.clone(),
                                license_id: internal_license_id,
                                artifact: product_artifact,
                                license_price_factor,
                            };

                            let new_cart = create_new_cart(new_cart_args).await;
                            tracing::debug!(
                                "new_cart(auth true, no existing cart): {:?}",
                                new_cart
                            );

                            new_cart
                        }
                    }
                }
                Err(_e) => {
                    let mut existing_cart_query = db
                        .query("SELECT * FROM cart WHERE archived=false AND session_id=$session_id LIMIT 1")
                        .bind(("session_id", session_id.clone()))
                        .await
                        .map_err(|e| Error::new(e.to_string()))?;

                    let existing_cart: Option<Cart> = existing_cart_query.take(0)?;

                    match existing_cart {
                        Some(cart) => {
                            let update_args = UpdateCartArgs {
                                cart: cart.clone(),
                                internal_product_id,
                                cart_operation,
                                product_price,
                                db_ctx: db.clone(),
                                license_id: internal_license_id,
                                artifact: product_artifact,
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
                                internal_product_id,
                                product_price,
                                internal_user_id: None,
                                db_ctx: db.clone(),
                                session_id: session_id.clone(),
                                license_id: internal_license_id,
                                artifact: product_artifact,
                                license_price_factor,
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
            Err(ExtendedError::new("Invalid Request!", Some(400.to_string())).build())
        }
    }
}

/// Utility function to update an instance of a cart
async fn update_existing_cart(args: UpdateCartArgs) -> Result<Cart> {
    let cart_id_raw = args.cart.id.as_ref().map(|t| &t.id).expect("id").to_raw();

    match args.cart_operation {
        CartOperation::AddProduct => {
            let mut update_cart_transaction = args.db_ctx
            .query(
                "
                BEGIN TRANSACTION;
                LET $product = type::thing($product_id);
                LET $cart = type::thing($cart_id);
                LET $cart_product = (SELECT * FROM cart_product WHERE out = $product AND in = $cart);
                LET $license = (SELECT id, price_factor FROM ONLY type::thing($license_id));
                LET $updated_quantity = IF array::len($cart_product) > 0
               	{

              		LET $found_product = $cart_product[0].id;
                    LET $prev_license_factor = (SELECT VALUE price_factor FROM ONLY $cart_product[0].license);

              		-- LET $updates = (UPDATE $found_product SET quantity += 1 RETURN AFTER);
                    LET $removed_amount = $prev_license_factor * $product_price;
                    UPDATE $cart SET total_amount -= $removed_amount RETURN AFTER;

              		LET $updates_license = (UPDATE $found_product SET license = $license.id RETURN AFTER);

              		RETURN $updates_license[0].quantity;

                }
                ELSE
               	{

              		LET $updates = (RELATE $cart -> cart_product -> $product CONTENT {
             			in: $cart,
             			license: $license.id,
             			out: $product,
             			quantity: 1,
                        artifact: $artifact
              		} RETURN AFTER);

              		RETURN $updates[0].quantity;

                }
                ;
                LET $total_amount = $product_price * $license.price_factor;
                LET $updated_cart = (UPDATE $cart SET total_amount += $total_amount RETURN AFTER);
                RETURN $updated_cart;
                COMMIT TRANSACTION;
                "
            )
            .bind(("product_price", args.product_price))
            .bind(("product_id", format!("product_id:{}", args.internal_product_id)))
            .bind(("cart_id", format!("cart:{}", cart_id_raw)))
            .bind(("license_id", format!("license:{}", args.license_id)))
            .bind(("artifact", args.artifact))
            .await
            .map_err(|e| Error::new(e.to_string()))?;

            let response: Vec<Cart> = update_cart_transaction.take(0)?;
            Ok(response.first().unwrap().to_owned())
        }
        CartOperation::RemoveProduct => {
            let mut update_cart_transaction = args.db_ctx
            .query(
                "
                BEGIN TRANSACTION;
                LET $product = type::thing($product_id);
                LET $cart = type::thing($cart_id);
                LET $license = (SELECT id, price_factor FROM ONLY type::thing($license_id));

                LET $product_exists = (SELECT quantity FROM cart_product WHERE out = $product AND in = $cart);
                IF $product_exists[0].quantity > 0 {
                    UPDATE $cart SET total_amount -= ($product_exists[0].quantity * $product_price * $license.price_factor);
                    DELETE $cart->cart_product WHERE out=$product;
                };

                LET $updated_cart = SELECT * FROM cart WHERE id=$cart;

                RETURN $updated_cart;
                COMMIT TRANSACTION;
                "
            )
            .bind(("product_price", args.product_price))
            .bind(("product_id", format!("product_id:{}", args.internal_product_id)))
            .bind(("cart_id", format!("cart:{}", cart_id_raw)))
            .bind(("license_id", format!("license:{}", args.license_id)))
            .await
            .map_err(|e| Error::new(e.to_string()))?;

            let response: Vec<Cart> = update_cart_transaction.take(0)?;
            Ok(response.first().unwrap().to_owned())
        }
    }
}

/// Utility function to create an instance of a cart
async fn create_new_cart(args: NewCartArgs) -> Result<Cart> {
    let mut create_cart_transaction = args
        .db_ctx
        .query(
            "
        BEGIN TRANSACTION;
        LET $product = type::thing($product_id);
        LET $owner = IF $user = '' {
            NONE
        } ELSE {
            type::thing($user)
        };
        LET $license = type::thing($license_id);

        LET $new_cart = (CREATE cart CONTENT {
           	owner: $owner,
           	total_amount: $product_price * $license_price_factor,
            session_id: $session_id
        });
        LET $cart_id = (SELECT VALUE id FROM $new_cart)[0];
        RELATE $cart_id-> cart_product -> $product CONTENT {
            quantity: 1,
            license: $license,
            artifact: $artifact
        };
        RETURN $new_cart;
        COMMIT TRANSACTION;
        ",
        )
        // .bind(("cart_product_details", cart_product_details))
        .bind(("license_price_factor", args.license_price_factor))
        .bind(("product_price", args.product_price))
        .bind((
            "product_id",
            format!("product_id:{}", args.internal_product_id),
        ))
        .bind((
            "user",
            match args.internal_user_id {
                Some(id) => format!("user_id:{}", id),
                None => "".to_string(),
            },
        ))
        .bind(("session_id", args.session_id))
        .bind(("license_id", format!("license_id:{}", args.license_id)))
        .bind(("artifact", args.artifact))
        .await
        .map_err(|e| Error::new(e.to_string()))?;

    let response: Vec<Cart> = create_cart_transaction.take(0)?;

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
        .map_err(|e| Error::new(e.to_string()))?;

    let existing_cart: Option<Cart> = existing_cart_query.take(0)?;

    Ok(existing_cart)
}
