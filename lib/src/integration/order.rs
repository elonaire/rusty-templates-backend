use std::{collections::HashMap, env, io::Error};

use hyper::HeaderMap;
use crate::utils::{graphql_api::perform_mutation_or_query_with_vars, models::{OrderStatus, UpdateOrderResponse, UpdateOrderVar}};

/// Integration method for Payments service, used across all the services
pub async fn update_order(headers: HeaderMap, order_id: String, status: OrderStatus) -> Result<String, Error> {
    // check auth status from ACL service(graphql query)
    let gql_query = r#"
        mutation OrderMutation($orderId: String!, $status: OrderStatus!) {
            updateOrder(orderId: $orderId, status: $status)
        }
    "#;

    let variables = UpdateOrderVar {
        order_id,
        status
    };

    match headers.get("Authorization") {
        Some(auth_header) => {
            let mut auth_headers = HashMap::new();
            auth_headers.insert("Authorization".to_string(), auth_header.to_str().unwrap().to_string());

            let endpoint = env::var("ORDERS_SERVICE")
            .expect("Missing the ORDERS_SERVICE environment variable.");

            let update_order_response = perform_mutation_or_query_with_vars::<UpdateOrderResponse, UpdateOrderVar>(Some(auth_headers), &endpoint, gql_query, variables).await;

            println!("update_order_response {:?}", update_order_response);

            match update_order_response.get_data() {
                Some(update_order_response) => {
                    println!("data here: {:?}", update_order_response);
                    Ok(update_order_response.update_order.clone())
                }
                None => {
                    Err(Error::new(std::io::ErrorKind::Other, format!("Orders service not responding! initiate_payment_integration")))
                }
            }
        }
        None => {
            Err(Error::new(std::io::ErrorKind::Other, "Not Authorized!"))
        }
    }
}
