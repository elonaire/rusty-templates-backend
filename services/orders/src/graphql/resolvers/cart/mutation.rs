use std::sync::Arc;

use crate::graphql::schemas::general::{Cart, CartOperation};
use async_graphql::{Context, Error, Object, Result};
use axum::{Extension, http::HeaderMap};
use surrealdb::{engine::remote::ws::Client, Surreal};
use lib::{integration::{auth::check_auth_from_acl, foreign_key::add_foreign_key_if_not_exists, product::get_product_price}, utils::{custom_error::ExtendedError, models::{ForeignKey, Product, User}}};

#[derive(Default)]
pub struct CartMutation;

#[Object]
impl CartMutation {
    pub async fn create_or_update_cart(&self, ctx: &Context<'_>, product_id: String, cart_operation: CartOperation) -> Result<Cart> {
        let db = ctx.data::<Extension<Arc<Surreal<Client>>>>().unwrap();

        if let Some(headers) = ctx.data_opt::<HeaderMap>() {
            let auth_status = check_auth_from_acl(headers.clone()).await?;

            let user_fk_body = ForeignKey {
                table: "user_id".into(),
                column: "user_id".into(),
                foreign_key: auth_status.decode_token
            };

            let product_fk_body = ForeignKey {
                table: "product_id".into(),
                column: "product_id".into(),
                foreign_key: product_id.clone()
            };
            let user_fk = add_foreign_key_if_not_exists::<User>(ctx, user_fk_body).await;
            let product_fk = add_foreign_key_if_not_exists::<Product>(ctx, product_fk_body).await;
            let user_id_raw = user_fk.unwrap().id.as_ref().map(|t| &t.id).expect("id").to_raw();
            let product_id_raw = product_fk.unwrap().id.as_ref().map(|t| &t.id).expect("id").to_raw();
            println!("product_id_raw: {}", product_id_raw);

            let mut existing_cart_query = db
                .query("SELECT * FROM cart WHERE archived=false AND owner=type::thing($user_id) LIMIT 1")
                .bind(("user_id", format!("user_id:{}", user_id_raw)))
                .await
                .map_err(|e| Error::new(e.to_string()))?;

            let existing_cart: Option<Cart> = existing_cart_query.take(0)?;

            let product_price = get_product_price(product_id.clone()).await?;

            match existing_cart {
                Some(cart) => {
                    let cart_id_raw = cart.id.as_ref().map(|t| &t.id).expect("id").to_raw();

                    match cart_operation {
                        CartOperation::AddProduct => {
                            let mut update_cart_transaction = db
                            .query(
                                "
                                BEGIN TRANSACTION;
                                LET $product = type::thing($product_id);
                                LET $cart = type::thing($cart_id);
                                LET $cart_product = (SELECT * FROM cart_product WHERE out = $product AND in = $cart);
                                LET $updated_quantity = IF array::len($cart_product) > 0
                               	{

                              		LET $found_product = $cart_product[0].id;

                              		LET $updates = (UPDATE $found_product SET quantity += 1 RETURN AFTER);

                              		RETURN $updates[0].quantity;

                                } ELSE {

                              		LET $updates = (RELATE $cart -> cart_product -> $product CONTENT {
                             			in: $cart,
                             			out: $product,
                             			quantity: 1
                              		} RETURN AFTER);

                              		RETURN $updates[0].quantity;

                                }
                                ;
                                -- LET $total_amount = $updated_quantity * $product_price;
                                LET $updated_cart = (UPDATE $cart SET total_amount += $product_price  RETURN AFTER);
                                RETURN $updated_cart;
                                COMMIT TRANSACTION;
                                "
                            )
                            .bind(("product_price", product_price))
                            .bind(("product_id", format!("product_id:{}", product_id_raw)))
                            .bind(("user", format!("user_id:{}", user_id_raw)))
                            .bind(("cart_id", format!("cart:{}", cart_id_raw)))
                            .await
                            .map_err(|e| Error::new(e.to_string()))?;

                            let response: Vec<Cart> = update_cart_transaction.take(0)?;
                            Ok(response.first().unwrap().to_owned())
                        },
                        CartOperation::RemoveProduct => {
                            let mut update_cart_transaction = db
                            .query(
                                "
                                BEGIN TRANSACTION;
                                LET $product = type::thing($product_id);
                                LET $cart = type::thing($cart_id);

                                LET $product_exists = (SELECT quantity FROM cart_product WHERE out = $product AND in = $cart);
                                IF $product_exists[0].quantity > 0 {
                                    UPDATE $cart SET total_amount -= ($product_exists[0].quantity * $product_price);
                                    DELETE $cart->cart_product WHERE out=$product;
                                };

                                LET $updated_cart = SELECT * FROM cart WHERE id=$cart;

                                RETURN $updated_cart;
                                COMMIT TRANSACTION;
                                "
                            )
                            .bind(("product_price", product_price))
                            .bind(("product_id", format!("product_id:{}", product_id_raw)))
                            .bind(("user", format!("user_id:{}", user_id_raw)))
                            .bind(("cart_id", format!("cart:{}", cart_id_raw)))
                            .await
                            .map_err(|e| Error::new(e.to_string()))?;

                            let response: Vec<Cart> = update_cart_transaction.take(0)?;
                            Ok(response.first().unwrap().to_owned())
                        }
                    }
                },
                None => {
                    let mut create_cart_transaction = db
                    .query(
                        "
                        BEGIN TRANSACTION;
                        LET $product = type::thing($product_id);
                        LET $new_cart = (CREATE cart CONTENT {
                           	owner: type::thing($user),
                           	total_amount: $product_price
                        });
                        LET $cart_id = (SELECT VALUE id FROM $new_cart)[0];
                        RELATE $cart_id-> cart_product -> $product CONTENT {
                            quantity: 1,
                            in: $cart_id,
                            out: $product
                        };
                        RETURN $new_cart;
                        COMMIT TRANSACTION;
                        "
                    )
                    // .bind(("cart_product_details", cart_product_details))
                    // .bind(("cart_details", cart_details))
                    .bind(("product_price", product_price))
                    .bind(("product_id", format!("product_id:{}", product_id_raw)))
                    .bind(("user", format!("user_id:{}", user_id_raw)))
                    .await
                    .map_err(|e| Error::new(e.to_string()))?;

                    let response: Vec<Cart> = create_cart_transaction.take(0)?;

                    Ok(response.first().unwrap().to_owned())
                }
            }
        } else {
            Err(ExtendedError::new("Cart is empty!", Some(400.to_string())).build())
        }
    }
}
