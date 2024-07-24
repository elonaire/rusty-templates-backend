use std::{collections::HashMap, env, io::Error};

use hyper::HeaderMap;
use crate::utils::{auth::DecodeTokenResponse, graphql_api::perform_query_without_vars};

pub async fn check_auth_from_acl(headers: HeaderMap) -> Result<DecodeTokenResponse, Error> {
    // check auth status from ACL service(graphql query)
    let gql_query = r#"
        mutation Mutation {
            decodeToken
        }
    "#;

    match headers.get("Authorization") {
        Some(auth_header) => {
            let mut auth_headers = HashMap::new();
            auth_headers.insert("Authorization".to_string(), auth_header.to_str().unwrap().to_string());

            let endpoint = env::var("OAUTH_SERVICE")
            .expect("Missing the OAUTH_SERVICE environment variable.");

            println!("OAUTH_SERVICE: {}", endpoint);

            // let client = GQLClient::new_with_headers(endpoint, auth_headers);

            let auth_response = perform_query_without_vars::<DecodeTokenResponse>(Some(auth_headers), endpoint.as_str(), gql_query).await;

            println!("auth_response: {:?}", auth_response);

            match auth_response.get_data() {
                Some(auth_response) => {
                    Ok(auth_response.to_owned())
                }
                None => {
                    Err(Error::new(std::io::ErrorKind::Other, "ACL server not responding! check_auth_from_acl"))
                }
            }
        }
        None => {
            Err(Error::new(std::io::ErrorKind::Other, "Not Authorized!"))
        }
    }
}
