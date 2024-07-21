use std::{collections::HashMap, env, io::Error};

use async_graphql::Context;
use gql_client::Client as GQLClient;
use hyper::HeaderMap;
use crate::utils::models::GetUserVar;

pub async fn get_user_email(ctx: &Context<'_>, user_id: String) -> Result<String, Error> {
    match ctx.data_opt::<HeaderMap>() {
        Some(headers) => {
            // check auth status from ACL service(graphql query)
            let gql_query = r#"
                query Query($id: String!) {
                    getUserEmail(id: $id)
                }
            "#;

            let variables = GetUserVar { id: user_id };

            match headers.get("Authorization") {
                Some(auth_header) => {
                    let mut auth_headers = HashMap::new();
                    auth_headers.insert("Authorization", auth_header.to_str().unwrap());

                    let endpoint = env::var("OAUTH_SERVICE")
                    .expect("Missing the OAUTH_SERVICE environment variable.");

                    let client = GQLClient::new_with_headers(endpoint, auth_headers);

                    let auth_response = client.query_with_vars::<String, GetUserVar>(gql_query, variables).await;

                    match auth_response {
                        Ok(auth_response) => {
                            Ok(auth_response.unwrap())
                        }
                        Err(_) => {
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
