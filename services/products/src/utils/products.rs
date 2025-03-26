use lib::utils::{custom_traits::AsSurrealClient, models::UploadedFile};
use std::io::{Error, ErrorKind};

use crate::graphql::schemas::general::{License, Product};

/// Utility function to get the price of a product by its ID.
pub async fn get_product_price<T: Clone + AsSurrealClient>(
    db: &T,
    product_id: &str,
) -> Result<u64, Error> {
    let response: Option<Product> = db
        .as_client()
        .select(("product", product_id))
        .await
        .map_err(|e| {
            tracing::error!("DB Query Failed: {}", e);
            Error::new(ErrorKind::Other, "DB Query Failed")
        })?;

    match response {
        Some(product) => Ok(product.price),
        None => Err(Error::new(ErrorKind::InvalidInput, "Invalid Request!")),
    }
}

/// Utility function to get the artifact of a product by its product ID and license ID.
pub async fn get_product_artifact<T: Clone + AsSurrealClient>(
    db: &T,
    product_id: &str,
    license_id: &str,
) -> Result<String, Error> {
    let mut product_artifact_query = db
        .as_client()
        .query(
            "
            BEGIN TRANSACTION;
            LET $product = type::thing($product_id);
            LET $license = type::thing($license_id);

            LET $file = SELECT * FROM ONLY file_id WHERE (<-(product_license_artifact WHERE license = $license)) LIMIT 1;
            RETURN $file;
            COMMIT TRANSACTION;
            "
        )
        .bind(("product_id", format!("product:{}", product_id)))
        .bind(("license_id", format!("license:{}", license_id)))
        .await
        .map_err(|e| {
            tracing::error!("DB Query Failed: {}", e);
            Error::new(ErrorKind::Other, "DB Query Failed")
        })?;

    let response: Option<UploadedFile> = product_artifact_query.take(0).map_err(|e| {
        tracing::error!("Deserialization Failed: {}", e);
        Error::new(ErrorKind::Other, "Deserialization Failed")
    })?;

    match response {
        Some(file) => Ok(file.file_id),
        None => Err(Error::new(
            ErrorKind::NotFound,
            "Product Artifact Not Found",
        )),
    }
}

pub async fn get_license_price_factor<T: Clone + AsSurrealClient>(
    db: &T,
    license_id: &str,
) -> Result<u64, Error> {
    let mut get_license_query = db
        .as_client()
        .query(
            "
            BEGIN TRANSACTION;
            LET $license_thing = type::thing($license_id);

            LET $license = SELECT * FROM ONLY $license_thing LIMIT 1;
            RETURN $license;
            COMMIT TRANSACTION;
            ",
        )
        .bind(("license_id", format!("license:{}", license_id)))
        .await
        .map_err(|e| {
            tracing::error!("DB Query Failed: {}", e);
            Error::new(ErrorKind::Other, "DB Query Failed")
        })?;

    let response: Option<License> = get_license_query.take(0).map_err(|e| {
        tracing::error!("Deserialization Failed: {}", e);
        Error::new(ErrorKind::Other, "Deserialization Failed")
    })?;

    match response {
        Some(license) => Ok(license.price_factor),
        None => Err(Error::new(ErrorKind::NotFound, "License Not Found")),
    }
}
