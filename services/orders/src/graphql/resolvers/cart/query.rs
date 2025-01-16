use std::sync::Arc;

use async_graphql::{Context, Error, Object, Result};
use axum::Extension;
use hyper::HeaderMap;
use lib::utils::custom_error::ExtendedError;
use surrealdb::{engine::remote::ws::Client, Surreal};

use crate::graphql::{resolvers::cart::mutation::set_session_cookie, schemas::general::Cart};

#[derive(Default)]
pub struct CartQuery;

#[Object]
impl CartQuery {
    async fn get_product_external_ids(
        &self,
        ctx: &Context<'_>,
        cart_id: String,
    ) -> Result<Vec<String>> {
        let db = ctx.data::<Extension<Arc<Surreal<Client>>>>().unwrap();

        let mut external_product_ids_query = db
        .query(
            "
            SELECT VALUE product_id FROM product_id WHERE id IN (SELECT VALUE out FROM cart_product WHERE in = type::thing($cart_id))
            "
        )
        .bind(("cart_id", format!("cart:{}", cart_id)))
        .await
        .map_err(|e| Error::new(e.to_string()))?;

        let response: Vec<String> = external_product_ids_query.take(0)?;

        Ok(response)
    }

    async fn get_cart(&self, ctx: &Context<'_>) -> Result<Cart> {
        let db = ctx.data::<Extension<Arc<Surreal<Client>>>>().unwrap();

        if let Some(headers) = ctx.data_opt::<HeaderMap>() {
            let session_id = set_session_cookie(&mut headers.clone(), ctx);
            let mut external_product_ids_query = db
                .query(
                    "
                SELECT * FROM ONLY cart WHERE session_id = $session_id AND archived=false LIMIT 1
                ",
                )
                .bind(("session_id", session_id))
                .await
                .map_err(|e| Error::new(e.to_string()))?;

            let response: Option<Cart> = external_product_ids_query.take(0)?;

            match response {
                Some(cart) => Ok(cart.clone()),
                None => Err(ExtendedError::new("Not found!", Some(404.to_string())).build()),
            }
            // Ok()
        } else {
            Err(ExtendedError::new("Invalid Request!", Some(400.to_string())).build())
        }
    }

    async fn get_product_total_sales(
        &self,
        ctx: &Context<'_>,
        external_product_id: String,
    ) -> Result<u64> {
        let db = ctx.data::<Extension<Arc<Surreal<Client>>>>().unwrap();

        let mut product_total_sales_query = db
        .query(
            "
            BEGIN TRANSACTION;
            LET $internal_product = (SELECT VALUE id FROM product_id WHERE product_id=$external_product_id);
            LET $total_sales = (SELECT VALUE count(<-cart_product<-(cart WHERE archived=true)) FROM ONLY $internal_product LIMIT 1);
            RETURN $total_sales;
            COMMIT TRANSACTION;
            "
        )
        .bind(("external_product_id", external_product_id))
        .await
        .map_err(|e| Error::new(e.to_string()))?;

        let response: Option<u64> = product_total_sales_query.take(0)?;

        match response {
            Some(total_sales) => Ok(total_sales),
            None => Err(ExtendedError::new("Not found!", Some(404.to_string())).build()),
        }
    }
}
