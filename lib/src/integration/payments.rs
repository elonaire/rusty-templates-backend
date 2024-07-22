use std::{collections::HashMap, env, io::Error};

use async_graphql::Context;
use gql_client::Client as GQLClient;
use hyper::HeaderMap;
use crate::utils::{graphql_api::perform_mutation_or_query_with_vars, models::{InitPaymentGraphQLResponse, InitializePaymentResponse, InitiatePaymentVar, UserPaymentDetails}};

/// Integration method for Payments service, used across all the services
pub async fn initiate_payment_integration(ctx: &Context<'_>, user_payment_details: UserPaymentDetails) -> Result<String, Error> {
    match ctx.data_opt::<HeaderMap>() {
        Some(headers) => {
            // check auth status from ACL service(graphql query)
            let gql_query = r#"
                mutation PaymentMutation($userPaymentDetails: UserPaymentDetailsInput!) {
                    initiatePayment(userPaymentDetails: $userPaymentDetails) {
                        data {
                            authorizationUrl
                        }
                    }
                }
            "#;

            let variables = InitiatePaymentVar { user_payment_details };

            match headers.get("Authorization") {
                Some(auth_header) => {
                    let mut auth_headers = HashMap::new();
                    auth_headers.insert("Authorization", auth_header.to_str().unwrap());

                    let endpoint = env::var("PAYMENTS_SERVICE")
                    .expect("Missing the PAYMENTS_SERVICE environment variable.");

                    // let client = GQLClient::new_with_headers(endpoint, auth_headers);

                    let payments_init_response = perform_mutation_or_query_with_vars::<InitPaymentGraphQLResponse, InitiatePaymentVar>(&endpoint, gql_query, variables).await;

                    match payments_init_response.get_data() {
                        Some(payments_init_response) => {
                            println!("data here: {:?}", payments_init_response);
                            Ok(payments_init_response.initiate_payment.data.authorization_url.clone())
                        }
                        None => {
                            Err(Error::new(std::io::ErrorKind::Other, format!("ACL server not responding!")))
                        }
                    }
                }
                None => {
                    Err(Error::new(std::io::ErrorKind::Other, "Not Authorized!"))
                }
            }
        }
        None => Err(Error::new(std::io::ErrorKind::Other, "Not Authorized!")),
    }
}
