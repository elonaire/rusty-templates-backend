use std::{
    env,
    io::{Error, ErrorKind},
};

use lib::{
    integration::grpc::clients::products_service::{
        products_service_client::ProductsServiceClient, ProductSkuIds,
    },
    utils::grpc::{create_grpc_client, AuthMetaData},
};
use tonic::transport::Channel;

use hyper::{
    header::{AUTHORIZATION, COOKIE},
    HeaderMap,
};

use crate::graphql::schemas::general::CartProduct;

/// A utility method to calculate the total amount a cart is holding
pub async fn calculate_cart_total_amount(
    headers: &HeaderMap,
    products: Vec<CartProduct>,
) -> Result<u64, Error> {
    let auth_header = headers.get(AUTHORIZATION);
    let cookie_header = headers.get(COOKIE);

    let product_sku_ids = products
        .iter()
        .map(|product| product.product_sku_id.clone())
        .collect::<Vec<String>>();

    let mut request = tonic::Request::new(ProductSkuIds { product_sku_ids });

    let auth_metadata: AuthMetaData<ProductSkuIds> = AuthMetaData {
        auth_header,
        cookie_header,
        constructed_grpc_request: Some(&mut request),
    };

    let products_service_grpc = env::var("PRODUCTS_SERVICE_GRPC")
        .expect("Missing the PRODUCTS_SERVICE_GRPC environment variable.");

    if let Ok(mut products_grpc_client) = create_grpc_client::<
        ProductSkuIds,
        ProductsServiceClient<Channel>,
    >(&products_service_grpc, true, Some(auth_metadata))
    .await
    .map_err(|e| {
        tracing::error!("Failed to connect to Products service: {}", e);
        Error::new(
            ErrorKind::Other,
            "Failed to connect to Products service".to_string(),
        )
    }) {
        if let Ok(res) = products_grpc_client
            .retrieve_product_sku_prices(request)
            .await
        {
            tracing::debug!("files_grpc_client res: {:?}", res);
            let prices = res.into_inner().prices.to_vec();

            tracing::debug!("prices: {:?}", prices);

            let sum = products
                .iter()
                .map(|product| {
                    // Find the matching ProductSkuPrice in the prices vector
                    let price = prices
                        .iter()
                        .find(|price| price.product_sku == product.product_sku_id)
                        .map(|price| price.unit_price)
                        .unwrap_or(0); // Default to 0 if no match is found
                    price * product.quantity as u64
                })
                .sum();

            Ok(sum)
        } else {
            Err(Error::new(
                ErrorKind::Other,
                "Failed to retrieve product prices".to_string(),
            ))
        }
    } else {
        Err(Error::new(
            ErrorKind::Other,
            "Failed to connect to Products service".to_string(),
        ))
    }
}
