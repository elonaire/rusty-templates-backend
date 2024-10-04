use std::sync::Arc;

use crate::graphql::schemas::general::{Cart, CartOperation};
use async_graphql::{Context, Error, Object, Result};
use axum::{http::HeaderMap, Extension};
use hyper::header::SET_COOKIE;
use lib::{
    integration::{
        auth::check_auth_from_acl, file::get_product_artifact,
        foreign_key::add_foreign_key_if_not_exists, product::get_product_price,
    },
    utils::{
        custom_error::ExtendedError,
        models::{ForeignKey, Product, User},
    },
};
use surrealdb::{engine::remote::ws::Client, Surreal};
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
}

#[derive(Default)]
pub struct CartMutation;

#[Object]
impl CartMutation {
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

            let product_fk = add_foreign_key_if_not_exists::<Product>(ctx, product_fk_body).await;
            let internal_product_id = product_fk
                .unwrap()
                .id
                .as_ref()
                .map(|t| &t.id)
                .expect("id")
                .to_raw();
            let product_price = get_product_price(external_product_id.clone()).await?;

            let product_artifact = get_product_artifact(
                &headers,
                external_product_id.clone(),
                external_license_id.clone(),
            )
            .await?;

            println!("product_artifact: {:?}", product_artifact);

            match check_auth_from_acl(headers.clone()).await {
                Ok(auth_status) => {
                    let user_fk_body = ForeignKey {
                        table: "user_id".into(),
                        column: "user_id".into(),
                        foreign_key: auth_status.sub,
                    };

                    let user_fk = add_foreign_key_if_not_exists::<User>(ctx, user_fk_body).await;

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
                                internal_product_id: internal_product_id.clone(),
                                cart_operation: cart_operation.clone(),
                                product_price,
                                db_ctx: db.clone(),
                                license_id: external_license_id.clone(),
                                artifact: product_artifact.clone(),
                            };

                            let updated_cart = update_existing_cart(update_args).await;

                            println!("{:?}", updated_cart);

                            updated_cart
                        }
                        None => {
                            println!("SHould Go here!");
                            let new_cart_args = NewCartArgs {
                                internal_product_id: internal_product_id.clone(),
                                product_price,
                                internal_user_id: Some(internal_user_id.clone()),
                                db_ctx: db.clone(),
                                session_id: session_id.clone(),
                                license_id: external_license_id.clone(),
                                artifact: product_artifact.clone(),
                            };

                            let new_cart = create_new_cart(new_cart_args).await;

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
                                internal_product_id: internal_product_id.clone(),
                                cart_operation: cart_operation.clone(),
                                product_price,
                                db_ctx: db.clone(),
                                license_id: external_license_id.clone(),
                                artifact: product_artifact.clone(),
                            };

                            let updated_cart = update_existing_cart(update_args).await;

                            updated_cart
                        }
                        None => {
                            let new_cart_args = NewCartArgs {
                                internal_product_id: internal_product_id.clone(),
                                product_price,
                                internal_user_id: None,
                                db_ctx: db.clone(),
                                session_id: session_id.clone(),
                                license_id: external_license_id.clone(),
                                artifact: product_artifact.clone(),
                            };

                            let new_cart = create_new_cart(new_cart_args).await;

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

async fn create_new_cart(args: NewCartArgs) -> Result<Cart> {
    println!("args.internal_user_id: {:?}", args);
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
        LET $license = (SELECT id, price_factor FROM ONLY type::thing($license_id));

        LET $new_cart = (CREATE cart CONTENT {
           	owner: $owner,
           	total_amount: $product_price * $license.price_factor,
            session_id: $session_id
        });
        LET $cart_id = (SELECT VALUE id FROM $new_cart)[0];
        RELATE $cart_id-> cart_product -> $product CONTENT {
            quantity: 1,
            in: $cart_id,
            out: $product,
            license: $license.id,
            artifact: $artifact
        };
        RETURN $new_cart;
        COMMIT TRANSACTION;
        ",
        )
        // .bind(("cart_product_details", cart_product_details))
        // .bind(("cart_details", cart_details))
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
        .bind(("license_id", format!("license:{}", args.license_id)))
        .bind(("artifact", args.artifact))
        .await
        .map_err(|e| Error::new(e.to_string()))?;

    let response: Vec<Cart> = create_cart_transaction.take(0)?;

    Ok(response.first().unwrap().to_owned())
}

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
