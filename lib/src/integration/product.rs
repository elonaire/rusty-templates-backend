use hyper::HeaderMap;
use std::{collections::HashMap, env, io::Error};

use crate::utils::{
    graphql_api::perform_mutation_or_query_with_vars,
    models::{
        GetLicensePriceFactorResponse, GetLicensePriceFactorVar, GetProductArtifactResponse,
        GetProductArtifactVar, GetProductPriceResponse, GetProductPriceVar,
    },
};

/// Get product price integration method
pub async fn get_product_price(product_id: String) -> Result<u64, Error> {
    let gql_query = r#"
        query ProductQuery($productId: String!) {
            getProductPrice(productId: $productId)
        }
    "#;

    let variables = GetProductPriceVar { product_id };

    let endpoint =
        env::var("PRODUCTS_SERVICE").expect("Missing the PRODUCTS_SERVICE environment variable.");

    let price_response = perform_mutation_or_query_with_vars::<
        GetProductPriceResponse,
        GetProductPriceVar,
    >(None, endpoint.as_str(), gql_query, variables)
    .await;

    match price_response.get_data() {
        Some(price_response) => Ok(price_response.to_owned().get_product_price.clone()),
        None => Err(Error::new(
            std::io::ErrorKind::Other,
            format!("Products Service not responding!"),
        )),
    }
}

/// Integration method for Product Artifact, returns a file ID.
pub async fn get_product_artifact(
    headers: &HeaderMap,
    product_id: String,
    license_id: String,
) -> Result<String, Error> {
    // check auth status from ACL service(graphql query)
    let gql_query = r#"
        query GetProductArtifact($productId: String!, $licenseId: String!) {
            getProductArtifact(productId: $productId, licenseId: $licenseId)
        }
    "#;

    let variables = GetProductArtifactVar {
        product_id,
        license_id,
    };

    match headers.get("Authorization") {
        Some(auth_header) => {
            let mut auth_headers = HashMap::new();
            auth_headers.insert(
                "Authorization".to_string(),
                auth_header.to_str().unwrap().to_string(),
            );
            if let Some(cookie_header) = headers.get("Cookie") {
                auth_headers.insert(
                    "Cookie".to_string(),
                    cookie_header.to_str().unwrap().to_string(),
                );
            };

            let endpoint = env::var("PRODUCTS_SERVICE")
                .expect("Missing the PRODUCTS_SERVICE environment variable.");

            let get_product_artifact =
                perform_mutation_or_query_with_vars::<
                    GetProductArtifactResponse,
                    GetProductArtifactVar,
                >(Some(auth_headers), &endpoint, gql_query, variables)
                .await;

            match get_product_artifact.get_data() {
                Some(get_product_artifact) => Ok(get_product_artifact.get_product_artifact.clone()),
                None => Err(Error::new(
                    std::io::ErrorKind::Other,
                    format!("Products service not responding! get_product_artifact"),
                )),
            }
        }
        None => Err(Error::new(std::io::ErrorKind::Other, "Not Authorized!")),
    }
}

/// Integration method for Product Price Factor, returns u64.
pub async fn get_license_price_factor(
    headers: &HeaderMap,
    license_id: String,
) -> Result<u64, Error> {
    // check auth status from ACL service(graphql query)
    let gql_query = r#"
        query GetLicensePriceFactor($licenseId: String!) {
            getLicensePriceFactor(licenseId: $licenseId)
        }
    "#;

    let variables = GetLicensePriceFactorVar { license_id };

    match headers.get("Authorization") {
        Some(auth_header) => {
            let mut auth_headers = HashMap::new();
            auth_headers.insert(
                "Authorization".to_string(),
                auth_header.to_str().unwrap().to_string(),
            );
            if let Some(cookie_header) = headers.get("Cookie") {
                auth_headers.insert(
                    "Cookie".to_string(),
                    cookie_header.to_str().unwrap().to_string(),
                );
            };

            let endpoint = env::var("PRODUCTS_SERVICE")
                .expect("Missing the PRODUCTS_SERVICE environment variable.");

            let get_license_price_factor =
                perform_mutation_or_query_with_vars::<
                    GetLicensePriceFactorResponse,
                    GetLicensePriceFactorVar,
                >(Some(auth_headers), &endpoint, gql_query, variables)
                .await;

            match get_license_price_factor.get_data() {
                Some(get_license_price_factor) => {
                    Ok(get_license_price_factor.get_license_price_factor.clone())
                }
                None => Err(Error::new(
                    std::io::ErrorKind::Other,
                    format!("Products service not responding! get_license_price_factor"),
                )),
            }
        }
        None => Err(Error::new(std::io::ErrorKind::Other, "Not Authorized!")),
    }
}
