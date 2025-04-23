use lib::utils::{
    custom_traits::AsSurrealClient,
    models::{ProductSkuPrice, UploadedFile},
};
use std::io::{Error, ErrorKind};

use crate::graphql::schemas::general::License;

/// Utility function to get the price of a product by its ID.
pub async fn get_product_price<T: Clone + AsSurrealClient>(
    db: &T,
    product_sku_id: &str,
) -> Result<u64, Error> {
    let copied_product_sku_id = product_sku_id.to_string();

    let mut product_price_query = db
        .as_client()
        .query(
            "
            BEGIN TRANSACTION;
            LET $product_sku = type::thing('product_sku', $product_sku_id);
            LET $price = (SELECT VALUE price FROM ONLY product WHERE ->(product_sku WHERE id = $product_sku) LIMIT 1);
            RETURN $price;
            COMMIT TRANSACTION;
            "
        )
        .bind(("product_sku_id", copied_product_sku_id))
        .await
        .map_err(|e| {
            tracing::error!("DB Query Failed: {}", e);
            Error::new(ErrorKind::Other, "DB Query Failed")
        })?;

    let response: Option<u64> = product_price_query.take(0).map_err(|e| {
        tracing::error!("Deserialization Failed: {}", e);
        Error::new(ErrorKind::Other, "Deserialization Failed")
    })?;

    match response {
        Some(price) => Ok(price),
        None => Err(Error::new(ErrorKind::InvalidInput, "Invalid Request!")),
    }
}

/// Utility function to get the artifact of a product sku by its product ID and license ID.
pub async fn get_product_sku_artifact<T: Clone + AsSurrealClient>(
    db: &T,
    product_sku_id: &str,
) -> Result<String, Error> {
    let mut product_sku_artifact_query = db
        .as_client()
        .query(
            "
            BEGIN TRANSACTION;
            LET $product_sku = type::thing('product_sku', $product_sku_id);
            LET $file = SELECT artifact[*] FROM ONLY $product_sku;
            RETURN $file.artifact;
            COMMIT TRANSACTION;
            ",
        )
        .bind(("product_sku_id", product_sku_id.to_string()))
        // .bind(("license_id", format!("license:{}", license_id)))
        .await
        .map_err(|e| {
            tracing::error!("DB Query Failed: {}", e);
            Error::new(ErrorKind::Other, "DB Query Failed")
        })?;

    let response: Option<UploadedFile> = product_sku_artifact_query.take(0).map_err(|e| {
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

pub async fn retrieve_product_sku_prices<T: Clone + AsSurrealClient>(
    db: &T,
    product_sku_ids: Vec<String>,
) -> Result<Vec<ProductSkuPrice>, Error> {
    let mut total_amount_query = db
        .as_client()
        .query(
            "
            BEGIN TRANSACTION;
            LET $product_skus = <array<record<product_sku>>> $product_sku_ids;

            LET $prices = SELECT ->product_sku.license[0].price_factor * price AS unit_price, record::id(->product_sku.id[0]) AS product_sku FROM product WHERE ->(product_sku WHERE id IN $product_skus);

            RETURN $prices;
            COMMIT TRANSACTION;
            ",
        )
        .bind(("product_sku_ids", product_sku_ids.into_iter().map(|product_sku_id| format!("product_sku:{}", product_sku_id)).collect::<Vec<String>>()))
        .await
        .map_err(|e| {
            tracing::error!("DB Query Failed: {}", e);
            Error::new(ErrorKind::Other, "DB Query Failed")
        })?;

    let prices: Vec<ProductSkuPrice> = total_amount_query.take(0).map_err(|e| {
        tracing::error!("Deserialization Failed: {}", e);
        Error::new(ErrorKind::Other, "Deserialization Failed")
    })?;

    Ok(prices)
}
