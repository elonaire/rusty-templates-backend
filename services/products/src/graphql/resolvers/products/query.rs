use std::sync::Arc;

use async_graphql::{Context, Error, Object, Result};
use axum::Extension;
use lib::utils::custom_error::ExtendedError;
use surrealdb::{engine::remote::ws::Client, Surreal};

use crate::{
    graphql::schemas::general::{License, Product},
    utils::products::{get_license_price_factor, get_product_artifact, get_product_price},
};

#[derive(Default)]
pub struct ProductQuery;

#[Object]
impl ProductQuery {
    async fn get_product_price(&self, ctx: &Context<'_>, product_id: String) -> Result<u64> {
        let db = ctx.data::<Extension<Arc<Surreal<Client>>>>().unwrap();

        let response = get_product_price(db, product_id.as_str()).await?;

        Ok(response)
    }

    async fn get_products(&self, ctx: &Context<'_>) -> Result<Vec<Product>> {
        let db = ctx.data::<Extension<Arc<Surreal<Client>>>>().unwrap();

        let products: Vec<Product> = db
            .select("product")
            .await
            .map_err(|e| Error::new(e.to_string()))?;

        Ok(products)
    }

    async fn get_products_by_ids(
        &self,
        ctx: &Context<'_>,
        product_ids: Vec<String>,
    ) -> Result<Vec<Product>> {
        let db = ctx.data::<Extension<Arc<Surreal<Client>>>>().unwrap();

        let records = product_ids
            .iter()
            .map(|product_id| format!("product:{}", product_id))
            .collect::<Vec<String>>();

        // let mut records_iter = records.iter().enumerate();
        let mut products: Vec<Product> = vec![];

        for (_, record) in records.iter().enumerate() {
            // Clone the record to own the String value for the query
            let mut products_query = db
                .query(
                    "
                        SELECT * FROM ONLY type::thing($product_id)
                        ",
                )
                .bind(("product_id", record.clone())) // Clone the record for ownership
                .await
                .map_err(|e| Error::new(e.to_string()))?;

            let product: Option<Product> = products_query.take(0)?;

            if let Some(p) = product {
                products.push(p);
            }
        }

        // while let Some(record) = records_iter.next() {
        //     let mut products_query = db
        //         .query(
        //             "
        //             SELECT * FROM ONLY type::thing($product_id)
        //             ",
        //         )
        //         .bind(("product_id", record.1))
        //         .await
        //         .map_err(|e| Error::new(e.to_string()))?;

        //     let product: Option<Product> = products_query.take(0)?;

        //     match product {
        //         Some(p) => {
        //             products.push(p);
        //         }
        //         None => {}
        //     }
        // }

        Ok(products)
    }

    async fn get_product_by_slug(&self, ctx: &Context<'_>, slug: String) -> Result<Product> {
        let db = ctx.data::<Extension<Arc<Surreal<Client>>>>().unwrap();

        let mut query_response = db
            .query(
                "
                SELECT * FROM ONLY product WHERE slug = $slug LIMIT 1
                ",
            )
            .bind(("slug", slug))
            .await
            .map_err(|e| Error::new(e.to_string()))?;

        let product: Option<Product> = query_response.take(0)?;

        match product {
            Some(product) => Ok(product),
            None => Err(ExtendedError::new("Product not found!", Some(404.to_string())).build()),
        }
    }

    pub async fn get_product_artifact(
        &self,
        ctx: &Context<'_>,
        product_id: String,
        license_id: String,
    ) -> Result<String> {
        let db = ctx.data::<Extension<Arc<Surreal<Client>>>>().unwrap();

        let response = get_product_artifact(db, product_id.as_str(), license_id.as_str()).await?;

        Ok(response)
    }

    pub async fn get_license_price_factor(
        &self,
        ctx: &Context<'_>,
        license_id: String,
    ) -> Result<u64> {
        let db = ctx.data::<Extension<Arc<Surreal<Client>>>>().unwrap();

        let response = get_license_price_factor(db, license_id.as_str()).await?;

        Ok(response)
    }

    async fn get_licenses(&self, ctx: &Context<'_>) -> Result<Vec<License>> {
        let db = ctx.data::<Extension<Arc<Surreal<Client>>>>().unwrap();

        let mut external_product_ids_query = db
            .query(
                "
            SELECT * FROM license ORDER BY price_factor ASC
            ",
            )
            .await
            .map_err(|e| Error::new(e.to_string()))?;

        let response: Vec<License> = external_product_ids_query.take(0)?;

        Ok(response)
    }
}
