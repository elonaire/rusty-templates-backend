use std::sync::Arc;

use async_graphql::{Context, Error, Object, Result};
use axum::Extension;
use hyper::HeaderMap;
use lib::utils::custom_error::ExtendedError;
use surrealdb::{engine::remote::ws::Client, Surreal};

use crate::graphql::{schemas::general::Cart, resolvers::cart::mutation::set_session_cookie};

#[derive(Default)]
pub struct CartQuery;

#[Object]
impl CartQuery {
    async fn get_product_external_ids(&self, ctx: &Context<'_>, cart_id: String) -> Result<Vec<String>> {
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
                SELECT * FROM cart WHERE session_id = $session_id
                "
            )
            .bind(("session_id", session_id))
            .await
            .map_err(|e| Error::new(e.to_string()))?;

            let response: Vec<Cart> = external_product_ids_query.take(0)?;

            match response.iter().nth(0) {
                Some(cart) => Ok(cart.clone()),
                None => Err(ExtendedError::new("Not found!", Some(404.to_string())).build())
            }
            // Ok()
        } else {
            Err(ExtendedError::new("Invalid Request!", Some(400.to_string())).build())
        }
    }
}
