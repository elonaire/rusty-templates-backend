use std::sync::Arc;

use crate::graphql::schemas::general::{Order, Cart};
use async_graphql::{Context, Error, Object, Result};
use axum::Extension;
use surrealdb::{engine::remote::ws::Client, Surreal};
use lib::{integration::{auth::check_auth_from_acl, foreign_key::add_foreign_key_if_not_exists, payments::initiate_payment_integration, user::get_user_email}, utils::{custom_error::ExtendedError, models::{ForeignKey, User, UserPaymentDetails}}};

#[derive(Default)]
pub struct OrderMutation;

#[Object]
impl OrderMutation {
    pub async fn create_order(&self, ctx: &Context<'_>, cart_id: String) -> Result<String> {
        let db = ctx.data::<Extension<Arc<Surreal<Client>>>>().unwrap();

        let auth_status = check_auth_from_acl(ctx).await?;

        let user_fk = ForeignKey {
            table: "user_id".into(),
            column: "user_id".into(),
            foreign_key: auth_status.decode_token.clone()
        };

        let buyer_result = add_foreign_key_if_not_exists::<User>(ctx, user_fk).await;
        let buyer_result_clone = buyer_result.clone();
        let buyer_id_raw = buyer_result_clone.unwrap().id.as_ref().map(|t| &t.id).expect("id").to_raw();

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

                match get_user_email(ctx, buyer_result.unwrap().user_id.clone()).await  {
                    Ok(email) => {
                        let payment_info = UserPaymentDetails {
                            email,
                            amount: 69,
                            // currency: None,
                            // metadata: None,
                        };
                        match initiate_payment_integration(ctx, payment_info).await {
                            Ok(payment_link) => {
                                Ok(payment_link)
                            },
                            Err(e) => Err(ExtendedError::new(format!("Error getting payment link! {:?}", e), Some(400.to_string())).build())
                        }
                    },
                    Err(e) => Err(ExtendedError::new(format!("User not found! {:?}", e), Some(400.to_string())).build())
                }

                // Ok(response)
            },
            None => Err(ExtendedError::new("Cart is empty!", Some(400.to_string())).build())
        }
    }
}
