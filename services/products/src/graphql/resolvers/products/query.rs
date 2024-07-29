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

    async fn get_products_by_ids(&self, ctx: &Context<'_>, product_ids: Vec<String>) -> Result<Vec<Product>> {
        let db = ctx.data::<Extension<Arc<Surreal<Client>>>>().unwrap();

        let records = product_ids.iter().map(|product_id| format!("product:{}", product_id)).collect::<Vec<String>>();

        let mut records_iter = records.iter().enumerate();
        let mut products: Vec<Product> = vec![];

        while let Some(record) = records_iter.next() {
            let mut products_query = db
                .query(
                    "
                    SELECT * FROM ONLY type::thing($product_id)
                    "
                )
                .bind(("product_id", record.1))
                .await
                .map_err(|e| Error::new(e.to_string()))?;

            let product: Option<Product> = products_query.take(0)?;

            match product {
                Some(p) => {
                    products.push(p);
                },
                None => {}
            }

        }

        Ok(products)
    }
}
