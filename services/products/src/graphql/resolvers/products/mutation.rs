use std::sync::Arc;

use crate::graphql::schemas::general::Product;
use async_graphql::{Context, Error, Object, Result};
use axum::{http::HeaderMap, Extension};
use lib::{
    integration::{auth::check_auth_from_acl, foreign_key::add_foreign_key_if_not_exists},
    utils::{
        custom_error::ExtendedError,
        models::{ForeignKey, User},
    },
};
use surrealdb::{engine::remote::ws::Client, Surreal};

#[derive(Default)]
pub struct ProductMutation;

#[Object]
impl ProductMutation {
    pub async fn create_product(
        &self,
        ctx: &Context<'_>,
        product: Product,
    ) -> Result<Vec<Product>> {
        let db = ctx.data::<Extension<Arc<Surreal<Client>>>>().unwrap();

        if let Some(headers) = ctx.data_opt::<HeaderMap>() {
            let auth_status = check_auth_from_acl(headers.clone()).await?;

            let foreign_key = ForeignKey {
                table: "user_id".into(),
                column: "user_id".into(),
                foreign_key: auth_status.sub,
            };
            let owner_result = add_foreign_key_if_not_exists::<User>(ctx, foreign_key).await;

            match owner_result {
                Some(owner) => {
                    let created_product: Vec<Product> = db
                        .create("product")
                        .content(Product {
                            owner: owner.id,
                            ..product
                        })
                        .await
                        .map_err(|e| Error::new(e.to_string()))?
                        .expect("Error creating product");

                    return Ok(created_product);
                }
                None => Err(ExtendedError::new("Not Authorized!", Some(403.to_string())).build()),
            }
        } else {
            Err(ExtendedError::new("Not Authorized!", Some(403.to_string())).build())
        }
    }
}
