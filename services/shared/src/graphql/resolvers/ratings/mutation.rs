use std::sync::Arc;

use crate::graphql::schemas::ratings::Rating;
use async_graphql::{Context, Error, Object, Result};
use axum::{http::HeaderMap, Extension};
use lib::{
    integration::{auth::check_auth_from_acl, foreign_key::add_foreign_key_if_not_exists},
    utils::{
        custom_error::ExtendedError,
        models::{ForeignKey, Product, User},
    },
};
use surrealdb::{engine::remote::ws::Client, Surreal};

#[derive(Default)]
pub struct RatingMutation;

#[Object]
impl RatingMutation {
    pub async fn rate_product(
        &self,
        ctx: &Context<'_>,
        rating: Rating,
        product_id: String,
    ) -> Result<Vec<Rating>> {
        let db = ctx.data::<Extension<Arc<Surreal<Client>>>>().unwrap();

        if let Some(headers) = ctx.data_opt::<HeaderMap>() {
            let auth_status = check_auth_from_acl(headers.clone()).await?;

            let user_fk = ForeignKey {
                table: "user_id".into(),
                column: "user_id".into(),
                foreign_key: auth_status.sub,
            };

            let product_fk = ForeignKey {
                table: "product_id".into(),
                column: "product_id".into(),
                foreign_key: product_id,
            };

            let author_result =
                add_foreign_key_if_not_exists::<Extension<Arc<Surreal<Client>>>, User>(db, user_fk)
                    .await;
            let rated_product_result = add_foreign_key_if_not_exists::<
                Extension<Arc<Surreal<Client>>>,
                Product,
            >(db, product_fk)
            .await;

            let mut rate_product_transaction = db
                .query(
                    "
                BEGIN TRANSACTION;
                LET $user = type::thing($user_id);
                LET $product = type::thing($product_id);
                LET $new_rating = (RELATE $user -> rating -> $product CONTENT {
                    rating_value: $rating_body.rating_value,
                } RETURN rating_value);
                RETURN $new_rating;
                COMMIT TRANSACTION;
                ",
                )
                .bind(("rating_body", rating))
                .bind((
                    "user_id",
                    format!(
                        "user_id:{}",
                        author_result
                            .unwrap()
                            .id
                            .as_ref()
                            .map(|t| &t.id)
                            .expect("id")
                            .to_raw()
                    ),
                ))
                .bind((
                    "product_id",
                    format!(
                        "product_id:{}",
                        rated_product_result
                            .unwrap()
                            .id
                            .as_ref()
                            .map(|t| &t.id)
                            .expect("id")
                            .to_raw()
                    ),
                ))
                .await
                .map_err(|e| Error::new(e.to_string()))?;

            let response: Vec<Rating> = rate_product_transaction.take(0).unwrap();

            Ok(response)
        } else {
            Err(ExtendedError::new("Not Authorized!", Some(403.to_string())).build())
        }
    }
}
