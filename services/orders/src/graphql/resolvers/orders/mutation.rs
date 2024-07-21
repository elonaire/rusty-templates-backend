use std::sync::Arc;

use crate::graphql::schemas::general::{Order, Cart};
use async_graphql::{Context, Error, Object, Result};
use axum::Extension;
use surrealdb::{engine::remote::ws::Client, Surreal};
use lib::{integration::{auth::check_auth_from_acl, foreign_key::add_foreign_key_if_not_exists}, utils::{models::{ForeignKey, User}, custom_error::ExtendedError}};

#[derive(Default)]
pub struct OrderMutation;

#[Object]
impl OrderMutation {
    pub async fn create_order(&self, ctx: &Context<'_>, cart_id: String) -> Result<Vec<Order>> {
        let db = ctx.data::<Extension<Arc<Surreal<Client>>>>().unwrap();

        let auth_res_from_acl = check_auth_from_acl(ctx).await?;

        match auth_res_from_acl {
            Some(auth_status) => {
                let user_fk = ForeignKey {
                    table: "user_id".into(),
                    column: "user_id".into(),
                    foreign_key: auth_status.decode_token.clone()
                };

                let buyer_result = add_foreign_key_if_not_exists::<User>(ctx, user_fk).await;
                let buyer_id_raw = buyer_result.unwrap().id.as_ref().map(|t| &t.id).expect("id").to_raw();

                let mut existing_cart_query = db
                    .query("SELECT * FROM cart WHERE archived=false AND owner=type::thing($user_id) LIMIT 1")
                    .bind(("user_id", format!("user_id:{}", buyer_id_raw)))
                    .await
                    .map_err(|e| Error::new(e.to_string()))?;

                let existing_cart: Option<Cart> = existing_cart_query.take(0)?;

                match existing_cart {
                    Some(_) => {
                        let mut create_order_transaction = db
                        .query(
                            "
                            BEGIN TRANSACTION;
                            LET $user = type::thing($user_id);
                            LET $cart = type::thing($cart_id);
                            LET $new_order = (RELATE $user -> order -> $cart CONTENT {
                                status: 'Pending',
                                in: $user,
                                out: $cart
                              } RETURN AFTER);
                            RETURN $new_order;
                            COMMIT TRANSACTION;
                            "
                        )
                        // .bind(("comment_body", comment))
                        .bind(("user_id", format!("user_id:{}", buyer_id_raw)))
                        .bind(("cart_id", format!("cart:{}", cart_id)))
                        .await
                        .map_err(|e| Error::new(e.to_string()))?;

                        let response: Vec<Order> = create_order_transaction.take(0).unwrap();

                        Ok(response)
                    },
                    None => Err(ExtendedError::new("Cart is empty!", Some(400.to_string())).build())
                }
            },
            None => Err(ExtendedError::new("Not Authorized!", Some(403.to_string())).build()),
        }
    }
}
