use std::sync::Arc;

use crate::graphql::schemas::general::Product;
use async_graphql::{Context, Error, Object, Result};
use axum::Extension;
use surrealdb::{engine::remote::ws::Client, Surreal};
use lib::{integration::{auth::check_auth_from_acl, foreign_key::add_foreign_key_if_not_exists}, utils::{models::{ForeignKey, User}, custom_error::ExtendedError}};

#[derive(Default)]
pub struct ProductMutation;

#[Object]
impl ProductMutation {
    pub async fn create_product(&self, ctx: &Context<'_>, product: Product) -> Result<Vec<Product>> {
        let db = ctx.data::<Extension<Arc<Surreal<Client>>>>().unwrap();

        let auth_res_from_acl = check_auth_from_acl(ctx).await?;

        match auth_res_from_acl {
            Some(auth_status) => {
                // let owner: Thing = format!("user_id:{}", auth_status.decode_token).parse().unwrap();
                let foreign_key = ForeignKey {
                    table: "user_id".into(),
                    column: "user_id".into(),
                    foreign_key: auth_status.decode_token
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
                            .map_err(|e| Error::new(e.to_string()))?;

                        Ok(created_product)
                    },
                    None => Err(ExtendedError::new("Not Authorized!", Some(403.to_string())).build()),
                }
            },
            None => Err(ExtendedError::new("Not Authorized!", Some(403.to_string())).build()),
        }
    }
}
