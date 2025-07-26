use std::sync::Arc;

use async_graphql::{Context, Object, Result};
use axum::Extension;
use hyper::StatusCode;
use lib::{
    integration::foreign_key::add_foreign_key_if_not_exists,
    utils::{
        custom_error::ExtendedError,
        models::{ForeignKey, Product},
    },
};
use surrealdb::{engine::remote::ws::Client, Surreal};

use crate::graphql::schemas::comments::Comment;

#[derive(Default)]
pub struct CommentsQuery;

#[Object]
impl CommentsQuery {
    async fn fetch_product_comments(
        &self,
        ctx: &Context<'_>,
        product_id: String,
    ) -> Result<Vec<Comment>> {
        let db = ctx.data::<Extension<Arc<Surreal<Client>>>>().map_err(|e| {
            tracing::error!("Error extracting Surreal Client: {:?}", e);
            ExtendedError::new(
                "Server Error",
                Some(StatusCode::INTERNAL_SERVER_ERROR.as_u16()),
            )
            .build()
        })?;

        let product_fk = ForeignKey {
            table: "product_id".into(),
            column: "product_id".into(),
            foreign_key: product_id,
        };

        let commented_product_result = add_foreign_key_if_not_exists::<
            Extension<Arc<Surreal<Client>>>,
            Product,
        >(db, product_fk)
        .await;

        if commented_product_result.is_none() {
            return Err(ExtendedError::new(
                "Product not found",
                Some(StatusCode::NOT_FOUND.as_u16()),
            )
            .build());
        }

        let mut comments_query = db
            .query(
                "
                BEGIN TRANSACTION;
                LET $product_id = type::thing($internal_product_id);
                LET $comments = SELECT * FROM comment WHERE <-has_comment<-(product_id WHERE id = $product_id);
                RETURN $comments;
                COMMIT TRANSACTION;
                ",
            )
            .bind(("internal_product_id", commented_product_result.unwrap().id))
            .await
            .map_err(|e| {
                tracing::error!("DB Query Error: {}", e);
                ExtendedError::new("Error fetching comments", Some(StatusCode::BAD_REQUEST.as_u16())).build()
            })?;

        let comments: Vec<Comment> = comments_query.take(0).map_err(|e| {
            tracing::error!("Deserialization Error: {}", e);
            ExtendedError::new(
                "Error fetching comments",
                Some(StatusCode::BAD_REQUEST.as_u16()),
            )
            .build()
        })?;
        Ok(comments)
    }
}
