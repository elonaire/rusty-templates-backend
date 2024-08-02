use std::{collections::HashMap, env, io::Error};

use hyper::HeaderMap;
use crate::utils::{graphql_api::perform_mutation_or_query_with_vars, models::{BuyProductArtifactResponse, BuyProductArtifactVar, GetProductArtifactResponse, GetProductArtifactVar}};

/// Integration method for Files service, used across all the services
pub async fn get_product_artifact(headers: &HeaderMap, external_product_id: String, external_license_id: String) -> Result<String, Error> {
    // check auth status from ACL service(graphql query)
    let gql_query = r#"
        query FileQuery($externalProductId: String!, $externalLicenseId: String!) {
            getProductArtifact(externalProductId: $externalProductId, externalLicenseId: $externalLicenseId)
        }
    "#;

    let variables = GetProductArtifactVar {
        external_product_id,
        external_license_id
    };

    match headers.get("Authorization") {
        Some(auth_header) => {
            let mut auth_headers = HashMap::new();
            auth_headers.insert("Authorization".to_string(), auth_header.to_str().unwrap().to_string());
            if let Some(cookie_header) =  headers.get("Cookie") {
                auth_headers.insert("Cookie".to_string(), cookie_header.to_str().unwrap().to_string());
            };

            let endpoint = env::var("FILES_SERVICE")
            .expect("Missing the FILES_SERVICE environment variable.");

            let get_product_artifact = perform_mutation_or_query_with_vars::<GetProductArtifactResponse, GetProductArtifactVar>(Some(auth_headers), &endpoint, gql_query, variables).await;

            println!("get_product_artifact: {:?}", get_product_artifact);

            match get_product_artifact.get_data() {
                Some(get_product_artifact) => {
                    Ok(get_product_artifact.get_product_artifact.clone())
                }
                None => {
                    Err(Error::new(std::io::ErrorKind::Other, format!("Files service not responding! get_product_artifact")))
                }
            }
        }
        None => {
            Err(Error::new(std::io::ErrorKind::Other, "Not Authorized!"))
        }
    }
}

pub async fn purchase_product_artifact(headers: &HeaderMap, file_name: String, ext_user_id: String) -> Result<String, Error> {
    // check auth status from ACL service(graphql query)
    let gql_query = r#"
        mutation FileMutation($fileName: String!, $extUserId: String!) {
            buyProductArtifactWebhook(fileName: $fileName, extUserId: $extUserId) {
                name
                size
                mimeType
                systemFilename
                isFree
                createdAt
                id
            }
        }
    "#;

    let variables = BuyProductArtifactVar {
        file_name,
        ext_user_id
    };

    match headers.get("Authorization") {
        Some(auth_header) => {
            let mut auth_headers = HashMap::new();
            auth_headers.insert("Authorization".to_string(), auth_header.to_str().unwrap().to_string());
            if let Some(cookie_header) =  headers.get("Cookie") {
                auth_headers.insert("Cookie".to_string(), cookie_header.to_str().unwrap().to_string());
            };

            let endpoint = env::var("FILES_SERVICE")
            .expect("Missing the FILES_SERVICE environment variable.");

            let buy_product_artifact = perform_mutation_or_query_with_vars::<BuyProductArtifactResponse, BuyProductArtifactVar>(Some(auth_headers), &endpoint, gql_query, variables).await;

            println!("buy_product_artifact: {:?}", buy_product_artifact);

            match buy_product_artifact.get_data() {
                Some(buy_product_artifact) => {
                    Ok(buy_product_artifact.buy_product_artifact_webhook.clone())
                }
                None => {
                    Err(Error::new(std::io::ErrorKind::Other, format!("Files service not responding! buy_product_artifact")))
                }
            }
        }
        None => {
            Err(Error::new(std::io::ErrorKind::Other, "Not Authorized!"))
        }
    }
}
