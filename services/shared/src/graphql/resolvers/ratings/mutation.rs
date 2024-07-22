use std::sync::Arc;

use crate::graphql::schemas::ratings::Rating;
use async_graphql::{Context, Error, Object, Result};
use axum::Extension;
use surrealdb::{engine::remote::ws::Client, Surreal};
use lib::{integration::{auth::check_auth_from_acl, foreign_key::add_foreign_key_if_not_exists}, utils::{models::{ForeignKey, User, Product}, custom_error::ExtendedError}};

#[derive(Default)]
pub struct RatingMutation;

#[Object]
impl RatingMutation {
    pub async fn rate_product(&self, ctx: &Context<'_>, rating: Rating, product_id: String) -> Result<Vec<Rating>> {
        let db = ctx.data::<Extension<Arc<Surreal<Client>>>>().unwrap();

        let auth_status = check_auth_from_acl(ctx).await?;

        let user_fk = ForeignKey {
            table: "user_id".into(),
            column: "user_id".into(),
            foreign_key: auth_status.decode_token
        };

        let product_fk = ForeignKey {
            table: "product_id".into(),
            column: "product_id".into(),
            foreign_key: product_id
        };

        let author_result = add_foreign_key_if_not_exists::<User>(ctx, user_fk).await;
        let rated_product_result = add_foreign_key_if_not_exists::<Product>(ctx, product_fk).await;

        let mut rate_product_transaction = db
        .query(
            "
            BEGIN TRANSACTION;
            LET $user = type::thing($user_id);
            LET $product = type::thing($product_id);
            LET $new_rating = (RELATE $user -> rating -> $product CONTENT {
                rating_value: $rating_body.rating_value,
                in: $user,
                out: $product
              } RETURN rating_value);
            RETURN $new_rating;
            COMMIT TRANSACTION;
            "
        )
        .bind(("rating_body", rating))
        .bind(("user_id", format!("user_id:{}", author_result.unwrap().id.as_ref().map(|t| &t.id).expect("id").to_raw())))
        .bind(("product_id", format!("product_id:{}", rated_product_result.unwrap().id.as_ref().map(|t| &t.id).expect("id").to_raw())))
        .await
        .map_err(|e| Error::new(e.to_string()))?;

        let response: Vec<Rating> = rate_product_transaction.take(0).unwrap();

        Ok(response)
    }
}
