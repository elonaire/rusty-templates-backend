use std::sync::Arc;

use async_graphql::{Context, Error, Object, Result};
use axum::Extension;
use hyper::HeaderMap;
use lib::{integration::auth::check_auth_from_acl, utils::{custom_error::ExtendedError, models::ArtifactsPurchaseDetails}};
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

    pub async fn get_all_order_artifacts(&self, ctx: &Context<'_>, order_id: String) -> Result<ArtifactsPurchaseDetails> {
        let db = ctx.data::<Extension<Arc<Surreal<Client>>>>().unwrap();

        if let Some(headers) = ctx.data_opt::<HeaderMap>() {
            let _auth_status = check_auth_from_acl(headers.clone()).await?;

            let mut order_artifacts_query = db
                .query(
                    "
                    BEGIN TRANSACTION;
                    LET $order_id = type::thing($id);
                    LET $artifacts = SELECT VALUE artifact FROM cart_product WHERE in=(SELECT VALUE ->cart FROM ONLY $order_id LIMIT 1)[0];
                    RETURN $artifacts;
                    COMMIT TRANSACTION;
                    "
                )
                .bind(("id", format!("order:{}", order_id)))
                .await
                .map_err(|e| Error::new(e.to_string()))?;

            let artifacts: Vec<String> = order_artifacts_query.take(0)?;

            let mut buyer_id_query = db
                .query(
                    "
                    BEGIN TRANSACTION;
                    LET $order = type::thing($id);
                    LET $buyer = SELECT VALUE (<-user_id.user_id)[0] FROM ONLY $order LIMIT 1;
                    RETURN $buyer;
                    COMMIT TRANSACTION;
                    "
                )
                .bind(("id", format!("order:{}", order_id)))
                .await
                .map_err(|e| Error::new(e.to_string()))?;

            let buyer_id: Option<String> = buyer_id_query.take(0)?;

            let purchase_details = ArtifactsPurchaseDetails {
                buyer_id: buyer_id.unwrap_or("".to_string()),
                artifacts
            };

            Ok(purchase_details)
        } else {
            Err(ExtendedError::new("Cart is empty!", Some(400.to_string())).build())
        }
    }
}
