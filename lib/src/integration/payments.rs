use std::{collections::HashMap, env, io::Error};

use async_graphql::Context;
use gql_client::Client as GQLClient;
use hyper::HeaderMap;
use crate::utils::models::{InitializePaymentResponse, InitiatePaymentVar, UserPaymentDetails};

pub async fn initiate_payment_integration(ctx: &Context<'_>, user_payment_details: UserPaymentDetails) -> Result<String, Error> {
    match ctx.data_opt::<HeaderMap>() {
        Some(headers) => {
            // check auth status from ACL service(graphql query)
            let gql_query = r#"
                mutation PaymentMutation($userPaymentDetails: UserPaymentDetails!) {
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

                    let client = GQLClient::new_with_headers(endpoint, auth_headers);

                    let payments_init_response = client.query_with_vars::<InitializePaymentResponse, InitiatePaymentVar>(gql_query, variables).await;

                    match payments_init_response {
                        Ok(payments_init_response) => {
                            Ok(payments_init_response.unwrap().data.authorization_url)
                        }
                        Err(e) => {
                            Err(Error::new(std::io::ErrorKind::Other, format!("ACL server not responding! {:?}", e)))
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
