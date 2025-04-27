use std::sync::Arc;

use crate::graphql::schemas::reviews::Review;
use async_graphql::{Context, Object, Result};
use axum::{http::HeaderMap, Extension};
use lib::{
    integration::foreign_key::add_foreign_key_if_not_exists,
    middleware::auth::graphql::check_auth_from_acl,
    utils::{
        custom_error::ExtendedError,
        models::{ForeignKey, Product, User},
    },
};
use surrealdb::{engine::remote::ws::Client, Surreal};

#[derive(Default)]
pub struct ReviewMutation;

#[Object]
impl ReviewMutation {
    pub async fn create_product_review(
        &self,
        ctx: &Context<'_>,
        review: Review,
        product_id: String,
    ) -> Result<Vec<Review>> {
        let db = ctx.data::<Extension<Arc<Surreal<Client>>>>().unwrap();

        if let Some(headers) = ctx.data_opt::<HeaderMap>() {
            let auth_status = check_auth_from_acl(&headers).await?;

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

            let mut review_product_transaction = db
                .query(
                    "
                    BEGIN TRANSACTION;
                    LET $user = type::thing('user_id', $user_id);
                    LET $product = type::thing('product_id', $product_id);
                    LET $new_review = (RELATE $user -> review -> $product CONTENT {
                        rating: $review.rating
                    } RETURN AFTER);
                    LET $new_review_value_id = (SELECT VALUE id FROM $new_review);

                    IF $review.comment IS NOT NONE {
                        LET $new_comment = (CREATE comment CONTENT {
                            content: $review.comment
                        } RETURN AFTER);
                        LET $new_comment_id = (SELECT VALUE id FROM $new_comment);
                        LET $new_review_comment = (RELATE $new_review_value_id -> has_comment -> $new_comment_id RETURN AFTER);
                    };

                    RETURN $new_review;
                    COMMIT TRANSACTION;
                ",
                )
                .bind(("review", review))
                .bind((
                    "user_id",
                    author_result
                        .unwrap()
                        .id
                        .as_ref()
                        .map(|t| &t.id)
                        .expect("id")
                        .to_raw(),
                ))
                .bind((
                    "product_id",
                    rated_product_result
                        .unwrap()
                        .id
                        .as_ref()
                        .map(|t| &t.id)
                        .expect("id")
                        .to_raw(),
                ))
                .await
                .map_err(|e| {
                    tracing::error!("DB Query Error: {}", e);
                    ExtendedError::new("Review not created", Some(400.to_string())).build()
                })?;

            let response: Vec<Review> = review_product_transaction.take(0).map_err(|e| {
                tracing::error!("Deserialization Error: {}", e);
                ExtendedError::new("Review not created", Some(400.to_string())).build()
            })?;

            Ok(response)
        } else {
            Err(ExtendedError::new("Not Authorized!", Some(403.to_string())).build())
        }
    }
}
