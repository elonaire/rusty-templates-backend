use std::sync::Arc;

use async_graphql::{Context, Error, Object, Result};
use axum::Extension;
use hyper::HeaderMap;
use lib::{
    middleware::auth::graphql::check_auth_from_acl,
    utils::{
        custom_error::ExtendedError,
        models::{ArtifactsPurchaseDetails, OrderStatus},
    },
};
use surrealdb::{engine::remote::ws::Client, Surreal};

use crate::{graphql::schemas::general::CartProduct, utils::orders::get_all_artifacts_for_order};

#[derive(Default)]
pub struct OrderQuery;

#[Object]
impl OrderQuery {
    async fn get_raw_cart_products(
        &self,
        ctx: &Context<'_>,
        cart_id: String,
    ) -> Result<Vec<CartProduct>> {
        let db = ctx.data::<Extension<Arc<Surreal<Client>>>>().unwrap();

        let mut cart_products_query = db
        .query(
            "
            BEGIN TRANSACTION;
            LET $cart = type::thing($cart_id);
            LET $cart_products = (SELECT VALUE ->cart_product FROM ONLY $cart);
            LET $aggregated = (SELECT *, (->product_id.product_id)[0] AS ext_product_id FROM $cart_products);

            RETURN $aggregated;
            COMMIT TRANSACTION;
            "
        )
        .bind(("cart_id", format!("cart:{}", cart_id)))
        .await
        .map_err(|e| Error::new(e.to_string()))?;

        let response: Vec<CartProduct> = cart_products_query.take(0)?;

        Ok(response)
    }

    pub async fn get_all_order_artifacts(
        &self,
        ctx: &Context<'_>,
        order_id: String,
    ) -> Result<ArtifactsPurchaseDetails> {
        let db = ctx.data::<Extension<Arc<Surreal<Client>>>>().unwrap();

        if let Some(headers) = ctx.data_opt::<HeaderMap>() {
            let _auth_status = check_auth_from_acl(headers).await?;

            let artifacts = get_all_artifacts_for_order(db, order_id.as_str()).await?;
            Ok(artifacts)
        } else {
            Err(ExtendedError::new("Cart is empty!", Some(400.to_string())).build())
        }
    }

    pub async fn get_customer_orders_by_status(
        &self,
        ctx: &Context<'_>,
        status: OrderStatus,
    ) -> Result<Vec<CartProduct>> {
        let db = ctx.data::<Extension<Arc<Surreal<Client>>>>().unwrap();

        if let Some(headers) = ctx.data_opt::<HeaderMap>() {
            let auth_status = check_auth_from_acl(headers).await?;

            let mut customer_past_orders_query = db
                .query(
                    "
                    BEGIN TRANSACTION;
                    LET $internal_user = (SELECT VALUE id FROM ONLY user_id WHERE user_id=$user_id LIMIT 1);

                    LET $cart_products = (SELECT VALUE (->cart->cart_product)[0] FROM order WHERE status='Confirmed' AND in=$internal_user);
                    LET $combined = (SELECT *, (->product_id.product_id)[0] AS ext_product_id FROM $cart_products);
                    RETURN $combined;
                    COMMIT TRANSACTION;
                    "
                )
                .bind(("user_id", auth_status.sub))
                .bind(("status", status))
                .await
                .map_err(|e| Error::new(e.to_string()))?;

            let previous_orders: Vec<CartProduct> = customer_past_orders_query.take(0)?;

            Ok(previous_orders)
        } else {
            Err(ExtendedError::new("Cart is empty!", Some(400.to_string())).build())
        }
    }
}
