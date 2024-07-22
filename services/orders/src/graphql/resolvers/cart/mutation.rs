use std::{collections::HashMap, sync::Arc};

use crate::graphql::schemas::general::{Cart, CartProduct};
use async_graphql::{Context, Error, Object, Result};
use axum::Extension;
use surrealdb::{engine::remote::ws::Client, Surreal};
use lib::{integration::{auth::check_auth_from_acl, foreign_key::add_foreign_key_if_not_exists}, utils::{models::{ForeignKey, User, Product}, custom_error::ExtendedError}};

#[derive(Default)]
pub struct CartMutation;

#[Object]
impl CartMutation {
    pub async fn create_or_update_cart(&self, ctx: &Context<'_>, quantity: u32, product_id: String) -> Result<Cart> {
        let db = ctx.data::<Extension<Arc<Surreal<Client>>>>().unwrap();

        let auth_status = check_auth_from_acl(ctx).await?;

        let user_fk_body = ForeignKey {
            table: "user_id".into(),
            column: "user_id".into(),
            foreign_key: auth_status.decode_token
        };

        let product_fk_body = ForeignKey {
            table: "product_id".into(),
            column: "product_id".into(),
            foreign_key: product_id
        };
        let user_fk = add_foreign_key_if_not_exists::<User>(ctx, user_fk_body).await;
        let product_fk = add_foreign_key_if_not_exists::<Product>(ctx, product_fk_body).await;
        let user_id_raw = user_fk.unwrap().id.as_ref().map(|t| &t.id).expect("id").to_raw();

        let mut existing_cart_query = db
            .query("SELECT * FROM cart WHERE archived=false AND owner=type::thing($user_id) LIMIT 1")
            .bind(("user_id", format!("user_id:{}", user_id_raw)))
            .await
            .map_err(|e| Error::new(e.to_string()))?;

        let existing_cart: Option<Cart> = existing_cart_query.take(0)?;

        match existing_cart {
            Some(cart) => {
                let cart_product_details = CartProduct {
                    id: None,
                    quantity
                };
                let cart_id_raw = cart.id.as_ref().map(|t| &t.id).expect("id").to_raw();

                let mut _update_cart_transaction = db
                .query(
                    "
                    BEGIN TRANSACTION;
                    LET $product = type::thing($product_id);
                    LET $cart_id = type::thing($cart_id);
                    LET $updated_cart = RELATE $cart_id-> cart_product -> $product CONTENT {
                        quantity: $cart_product_details.quantity,
                        in: $cart_id,
                        out: $product
                    } RETURN AFTER;
                    RETURN $updated_cart;
                    COMMIT TRANSACTION;
                    "
                )
                .bind(("cart_product_details", cart_product_details))
                .bind(("product_id", format!("product_id:{}", product_fk.unwrap().id.as_ref().map(|t| &t.id).expect("id").to_raw())))
                .bind(("user", format!("user_id:{}", user_id_raw)))
                .bind(("cart_id", format!("cart:{}", cart_id_raw)))
                .await
                .map_err(|e| Error::new(e.to_string()))?;

                // let response: Vec<Cart> = update_cart_transaction.take(0)?;
                Ok(cart)
            },
            None => {
                let cart_product_details = CartProduct {
                    id: None,
                    quantity
                };

                // let cart_details = Cart {
                //     id: None,
                //     archived: None,
                // };
                let mut create_cart_transaction = db
                .query(
                    "
                    BEGIN TRANSACTION;
                    LET $product = type::thing($product_id);
                    LET $new_cart = (CREATE cart CONTENT {
                       	owner: type::thing($user),
                       	total_amount: 69
                    });
                    LET $cart_id = (SELECT VALUE id FROM $new_cart)[0];
                    RELATE $cart_id-> cart_product -> $product CONTENT {
                        quantity: $cart_product_details.quantity,
                        in: $cart_id,
                        out: $product
                    };
                    RETURN $new_cart;
                    COMMIT TRANSACTION;
                    "
                )
                .bind(("cart_product_details", cart_product_details))
                // .bind(("cart_details", cart_details))
                .bind(("product_id", format!("product_id:{}", product_fk.unwrap().id.as_ref().map(|t| &t.id).expect("id").to_raw())))
                .bind(("user", format!("user_id:{}", user_id_raw)))
                .await
                .map_err(|e| Error::new(e.to_string()))?;

                let response: Vec<Cart> = create_cart_transaction.take(0)?;

                Ok(response.first().unwrap().to_owned())
            }
        }
    }
}
