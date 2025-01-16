use std::{collections::HashMap, env, io::Error};

use crate::utils::{
    graphql_api::perform_mutation_or_query_with_vars,
    models::{GetUserResponse, GetUserVar},
};
use async_graphql::Context;
use hyper::HeaderMap;

/// Get User Email integration method
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

                    let endpoint = env::var("OAUTH_SERVICE")
                        .expect("Missing the OAUTH_SERVICE environment variable.");

                    let email_response =
                        perform_mutation_or_query_with_vars::<GetUserResponse, GetUserVar>(
                            Some(auth_headers),
                            endpoint.as_str(),
                            gql_query,
                            variables,
                        )
                        .await;

                    println!("email_response: {:?}", email_response);

                    match email_response.get_data() {
                        Some(email_response) => {
                            Ok(email_response.to_owned().get_user_email.clone())
                        }
                        None => Err(Error::new(
                            std::io::ErrorKind::Other,
                            format!("ACL server not responding!"),
                        )),
                    }
                }
                None => Err(Error::new(std::io::ErrorKind::Other, "Not Authorized!")),
            }
        }
        None => Err(Error::new(std::io::ErrorKind::Other, "Not Authorized!")),
    }
}
