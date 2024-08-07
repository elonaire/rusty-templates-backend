use std::{env, io::Error};

use crate::utils::{graphql_api::perform_mutation_or_query_with_vars, models::{GetProductPriceResponse, GetProductPriceVar}};

/// Get product price integration method
pub async fn get_product_price(product_id: String) -> Result<u64, Error> {
    let gql_query = r#"
        query ProductQuery($productId: String!) {
            getProductPrice(productId: $productId)
        }
    "#;

    let variables = GetProductPriceVar { product_id };

    let endpoint = env::var("PRODUCTS_SERVICE")
    .expect("Missing the PRODUCTS_SERVICE environment variable.");

    let price_response = perform_mutation_or_query_with_vars::<GetProductPriceResponse, GetProductPriceVar>(None, endpoint.as_str(), gql_query, variables).await;

    match price_response.get_data() {
        Some(price_response) => {
            Ok(price_response.to_owned().get_product_price.clone())
        }
        None => {
            Err(Error::new(std::io::ErrorKind::Other, format!("Products Service not responding!")))
        }
    }
}
