use std::{collections::HashMap, env, io::Error};

use async_graphql::Context;
use gql_client::Client as GQLClient;
use hyper::HeaderMap;
use crate::utils::{auth::DecodeTokenResponse, graphql_api::perform_query_without_vars};

pub async fn check_auth_from_acl(ctx: &Context<'_>) -> Result<DecodeTokenResponse, Error> {
    match ctx.data_opt::<HeaderMap>() {
        Some(headers) => {
            // check auth status from ACL service(graphql query)
            let gql_query = r#"
                mutation Mutation {
                    decodeToken
                }
            "#;

            match headers.get("Authorization") {
                Some(auth_header) => {
                    let mut auth_headers = HashMap::new();
                    auth_headers.insert("Authorization", auth_header.to_str().unwrap());

                    let endpoint = env::var("OAUTH_SERVICE")
                    .expect("Missing the OAUTH_SERVICE environment variable.");

                    // let client = GQLClient::new_with_headers(endpoint, auth_headers);

                    let auth_response = perform_query_without_vars::<DecodeTokenResponse>(endpoint.as_str(), gql_query).await;

                    println!("auth_response: {:?}", auth_response);

                    match auth_response.get_data() {
                        Some(auth_response) => {
                            Ok(auth_response.to_owned())
                        }
                        None => {
                            Err(Error::new(std::io::ErrorKind::Other, "ACL server not responding!"))
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
