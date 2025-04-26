use std::sync::Arc;

use crate::graphql::schemas::comments::Comment;
use async_graphql::{Context, Error, Object, Result};
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
pub struct CommentMutation;

#[Object]
impl CommentMutation {
    pub async fn post_comment(
        &self,
        ctx: &Context<'_>,
        comment: Comment,
        product_id: String,
    ) -> Result<Vec<Comment>> {
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
            let commented_product_result = add_foreign_key_if_not_exists::<
                Extension<Arc<Surreal<Client>>>,
                Product,
            >(db, product_fk)
            .await;

            let mut post_comment_transaction = db
                .query(
                    "
                BEGIN TRANSACTION;
                LET $user = type::thing('user_id', $user_id);
                LET $product = type::thing('product_id', $product_id);
                LET $new_comment = (CREATE comment CONTENT {
                    content: $comment_body.content,
                } RETURN AFTER);
                RELATE $user -> wrote -> $new_comment;
                RELATE $product -> has_comment -> $new_comment;
                RETURN $new_comment;
                COMMIT TRANSACTION;
                ",
                )
                .bind(("comment_body", comment))
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
                    commented_product_result
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
                    ExtendedError::new("Comment not posted", Some(400.to_string())).build()
                })?;

            let response: Vec<Comment> = post_comment_transaction.take(0).map_err(|e| {
                tracing::error!("Deserialization Error: {}", e);
                ExtendedError::new("Rating not created", Some(400.to_string())).build()
            })?;

            Ok(response)
        } else {
            Err(ExtendedError::new("Not Authorized!", Some(403.to_string())).build())
        }
    }

    /// Reply to a comment
    pub async fn reply_to_a_comment(
        &self,
        ctx: &Context<'_>,
        comment: Comment,
        comment_id: String,
    ) -> async_graphql::Result<Vec<Comment>> {
        let db = ctx.data::<Extension<Arc<Surreal<Client>>>>().unwrap();

        let headers = ctx.data::<HeaderMap>().unwrap();

        let auth_res_from_acl = check_auth_from_acl(headers).await?;

        let user_fk = ForeignKey {
            table: "user_id".to_string(),
            column: "user_id".to_string(),
            foreign_key: auth_res_from_acl.sub.clone(),
        };

        let user_id_added =
            add_foreign_key_if_not_exists::<Extension<Arc<Surreal<Client>>>, User>(db, user_fk)
                .await;

        if user_id_added.is_none() {
            return Err(ExtendedError::new("Failed to add user_id", Some(500.to_string())).build());
        }

        let mut database_transaction = db
            .query(
                "
            BEGIN TRANSACTION;
            LET $user = type::thing('user_id', $user_id);
            LET $product = type::thing('product_id', $product_id);
            LET $parent_comment = type::thing('comment', $parent_comment_id);

            LET $new_comment = (CREATE comment CONTENT {
                content: $comment_body.content,
            } RETURN AFTER);
            LET $comment_reply_id = (SELECT VALUE id FROM $new_comment);
            RELATE $parent_comment->has_reply->$comment_reply_id;
            RELATE $user -> wrote -> $new_comment;
            RETURN $new_comment;
            COMMIT TRANSACTION;
            ",
            )
            .bind(("comment_body", comment))
            .bind((
                "user_id",
                user_id_added
                    .unwrap()
                    .id
                    .as_ref()
                    .map(|t| &t.id)
                    .expect("id")
                    .to_raw(),
            ))
            .bind(("parent_comment_id", comment_id))
            .await
            .map_err(|e| {
                tracing::debug!("DB Query Error: {}", e);
                Error::new("Internal Server Error")
            })?;

        let response: Vec<Comment> = database_transaction.take(0).map_err(|_e| {
            tracing::debug!("database_transaction: {:?}", database_transaction);
            Error::new("Internal Server Error")
        })?;

        Ok(response)
    }
}
