use std::sync::Arc;

use async_graphql::{Context, Error, Object, Result};
use axum::Extension;
use hyper::HeaderMap;
use lib::utils::custom_error::ExtendedError;
use surrealdb::{engine::remote::ws::Client, Surreal};

use crate::graphql::{resolvers::cart::mutation::set_session_cookie, schemas::general::{Cart, CartProduct, License}};

#[derive(Default)]
pub struct OrderQuery;

#[Object]
impl OrderQuery {
    async fn get_licenses(&self, ctx: &Context<'_>) -> Result<Vec<License>> {
        let db = ctx.data::<Extension<Arc<Surreal<Client>>>>().unwrap();

        let mut external_product_ids_query = db
        .query(
            "
            SELECT * FROM license ORDER BY price_factor ASC
            "
        )
        .await
        .map_err(|e| Error::new(e.to_string()))?;

        let response: Vec<License> = external_product_ids_query.take(0)?;

        Ok(response)
    }

    async fn get_raw_cart_products(&self, ctx: &Context<'_>, cart_id: String) -> Result<Vec<CartProduct>> {
        let db = ctx.data::<Extension<Arc<Surreal<Client>>>>().unwrap();

        let mut cart_products_query = db
        .query(
            "
            SELECT *, (SELECT VALUE product_id FROM ONLY (SELECT VALUE out FROM cart_product WHERE in = type::thing($cart_id) AND id = $parent.id) LIMIT 1) AS ext_product_id FROM cart_product;
            "
        )
        .bind(("cart_id", format!("cart:{}", cart_id)))
        .await
        .map_err(|e| Error::new(e.to_string()))?;

        let response: Vec<CartProduct> = cart_products_query.take(0)?;

        Ok(response)
    }
}
