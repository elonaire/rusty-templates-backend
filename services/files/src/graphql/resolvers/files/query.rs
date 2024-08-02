use std::sync::Arc;

use async_graphql::{Context, Error, Object, Result};
use axum::Extension;
use hyper::HeaderMap;
use lib::utils::custom_error::ExtendedError;
use surrealdb::{engine::remote::ws::Client, Surreal};

use crate::graphql::schemas::general::UploadedFile;

#[derive(Default)]
pub struct FileQuery;

#[Object]
impl FileQuery {
    pub async fn get_product_artifact(&self, ctx: &Context<'_>, external_product_id: String, external_license_id: String) -> Result<UploadedFile> {
        let db = ctx.data::<Extension<Arc<Surreal<Client>>>>().unwrap();

        let mut product_artifact_query = db
        .query(
            "
            BEGIN TRANSACTION;
            LET $internal_product = (SELECT VALUE id FROM ONLY product_id WHERE product_id=$product_id LIMIT 1);
            LET $internal_license = (SELECT VALUE id FROM ONLY license_id WHERE license_id=$license_id LIMIT 1);

            LET $file = (SELECT * FROM ONLY file WHERE <-(product_license_artifact WHERE license=$internal_license AND in=$internal_product) LIMIT 1);

            RETURN $file;
            COMMIT TRANSACTION;
            "
        )
        .bind(("product_id", external_product_id))
        .bind(("license_id", external_license_id))
        .await
        .map_err(|e| Error::new(e.to_string()))?;

        let response: Option<UploadedFile> = product_artifact_query.take(0)?;

        match response {
            Some(file) => Ok(file),
            None => Err(ExtendedError::new("Invalid parameters!", Some(400.to_string())).build())
        }
    }
}
