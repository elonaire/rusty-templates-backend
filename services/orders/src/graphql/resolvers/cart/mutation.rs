use std::sync::Arc;

use crate::graphql::schemas::general::{Cart, CartProduct};
use async_graphql::{Context, Error, Object, Result};
use axum::Extension;
use surrealdb::{engine::remote::ws::Client, Surreal};
use lib::{middleware::{auth::check_auth_from_acl, foreign_key::add_foreign_key_if_not_exists}, utils::{models::{ForeignKey, User, Product}, custom_error::ExtendedError}};

#[derive(Default)]
pub struct CartMutation;

#[Object]
impl CartMutation {
    pub async fn create_or_update_cart(&self, ctx: &Context<'_>, quantity: u32, product_id: String) -> Result<Vec<Cart>> {
        let db = ctx.data::<Extension<Arc<Surreal<Client>>>>().unwrap();

        let auth_res_from_acl = check_auth_from_acl(ctx).await?;

        match auth_res_from_acl {
            Some(auth_status) => {
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
                    .query("SELECT ->(cart WHERE archived=false) FROM type::thing($user_id)")
                    .bind(("user_id", format!("user_id:{}", user_id_raw)))
                    .await
                    .map_err(|e| Error::new(e.to_string()))?;

                let existing_cart: Option<Vec<Cart>> = existing_cart_query.take(0)?;

                match existing_cart {
                    Some(cart) => {
                        Ok(cart)
                    },
                    None => {
                        let cart_product_details = CartProduct {
                            id: None,
                            quantity
                        };

                        let cart_details = Cart {
                            id: None,
                            archived: None,
                        };
                        let mut create_cart_transaction = db
                        .query(
                            "
                            BEGIN TRANSACTION;
                            LET $new_cart_product = CREATE cart_product CONTENT $cart_product_details;
                            LET $new_cart = CREATE cart CONTENT $cart_details;
                            LET $cart_id = (SELECT id FROM new_cart);

                            LET $cart_product_id = (SELECT VALUE id FROM $new_cart_product);
                            RELATE $user->cart->$cart_id;
                            RELATE $cart_id->cart_product->type::thing($product_id);
                            RETURN $new_cart;
                            COMMIT TRANSACTION;
                            "
                        )
                        .bind(("cart_product_details", cart_product_details))
                        .bind(("cart_details", cart_details))
                        .bind(("product_id", format!("product_id:{}", product_fk.unwrap().id.as_ref().map(|t| &t.id).expect("id").to_raw())))
                        .await
                        .map_err(|e| Error::new(e.to_string()))?;

                        let response: Vec<Cart> = create_cart_transaction.take(0).unwrap();

                        Ok(response)
                    }
                }
            },
            None => Err(ExtendedError::new("Not Authorized!", Some(403.to_string())).build())
        }
    }
}
