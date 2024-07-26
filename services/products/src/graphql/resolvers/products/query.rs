use std::sync::Arc;

use async_graphql::{Context, Error, Object, Result};
use axum::Extension;
use lib::utils::custom_error::ExtendedError;
use surrealdb::{engine::remote::ws::Client, Surreal};

use crate::graphql::schemas::general::Product;

#[derive(Default)]
pub struct ProductQuery;

#[Object]
impl ProductQuery {
    async fn get_product_price(&self, ctx: &Context<'_>, product_id: String) -> Result<u64> {
        let db = ctx.data::<Extension<Arc<Surreal<Client>>>>().unwrap();

        println!("Goes through ProductQuery");

        let response: Option<Product> = db
            .select(("product", product_id))
            .await
            .map_err(|e| Error::new(e.to_string()))?;

        match response {
            Some(product) => Ok(product.price),
            None => Err(ExtendedError::new("Invalid Request!", Some(400.to_string())).build())
        }
    }

    async fn get_products(&self, ctx: &Context<'_>) -> Result<Vec<Product>> {
        let db = ctx.data::<Extension<Arc<Surreal<Client>>>>().unwrap();

        let products: Vec<Product> = db
            .select("product")
            .await
            .map_err(|e| Error::new(e.to_string()))?;

        Ok(products)
    }
}
