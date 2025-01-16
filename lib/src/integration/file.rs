use std::{collections::HashMap, env, io::Error};

use crate::utils::{
    graphql_api::perform_mutation_or_query_with_vars,
    models::{
        BuyProductArtifactResponse, BuyProductArtifactVar, GetFileIdResponse, GetFileIdVar,
        GetFileNameResponse, GetFileNameVar,
    },
};
use hyper::HeaderMap;

pub async fn purchase_product_artifact(
    headers: &HeaderMap,
    file_name: String,
    ext_user_id: String,
) -> Result<String, Error> {
    // check auth status from ACL service(graphql query)
    let gql_query = r#"
        mutation FileMutation($fileName: String!, $extUserId: String!) {
            buyProductArtifactWebhook(fileName: $fileName, extUserId: $extUserId)
        }
    "#;

    let variables = BuyProductArtifactVar {
        file_name,
        ext_user_id,
    };

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

            let endpoint =
                env::var("FILES_SERVICE").expect("Missing the FILES_SERVICE environment variable.");

            let buy_product_artifact =
                perform_mutation_or_query_with_vars::<
                    BuyProductArtifactResponse,
                    BuyProductArtifactVar,
                >(Some(auth_headers), &endpoint, gql_query, variables)
                .await;

            println!("buy_product_artifact: {:?}", buy_product_artifact);

            match buy_product_artifact.get_data() {
                Some(buy_product_artifact) => {
                    Ok(buy_product_artifact.buy_product_artifact_webhook.clone())
                }
                None => Err(Error::new(
                    std::io::ErrorKind::Other,
                    format!("Files service not responding! buy_product_artifact"),
                )),
            }
        }
        None => Err(Error::new(std::io::ErrorKind::Other, "Not Authorized!")),
    }
}

pub async fn get_file_id(headers: &HeaderMap, file_name: String) -> Result<String, Error> {
    // check auth status from ACL service(graphql query)
    let gql_query = r#"
        query GetFileId($fileName: String!) {
            getFileId(fileName: $fileName)
        }
    "#;

    let variables = GetFileIdVar { file_name };

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

            let endpoint =
                env::var("FILES_SERVICE").expect("Missing the FILES_SERVICE environment variable.");

            let get_file_info = perform_mutation_or_query_with_vars::<
                GetFileIdResponse,
                GetFileIdVar,
            >(Some(auth_headers), &endpoint, gql_query, variables)
            .await;

            println!("get_file_info: {:?}", get_file_info);

            match get_file_info.get_data() {
                Some(get_file_info) => Ok(get_file_info.get_file_id.clone()),
                None => Err(Error::new(
                    std::io::ErrorKind::Other,
                    format!("Files service not responding! buy_product_artifact"),
                )),
            }
        }
        None => Err(Error::new(std::io::ErrorKind::Other, "Not Authorized!")),
    }
}

pub async fn get_file_name(headers: &HeaderMap, file_id: String) -> Result<String, Error> {
    // check auth status from ACL service(graphql query)
    let gql_query = r#"
        query GetFileName($fileId: String!) {
            getFileName(fileId: $fileId)
        }
    "#;

    let variables = GetFileNameVar { file_id };

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

            let endpoint =
                env::var("FILES_SERVICE").expect("Missing the FILES_SERVICE environment variable.");

            let get_file_info = perform_mutation_or_query_with_vars::<
                GetFileNameResponse,
                GetFileNameVar,
            >(Some(auth_headers), &endpoint, gql_query, variables)
            .await;

            println!("get_file_info: {:?}", get_file_info);

            match get_file_info.get_data() {
                Some(get_file_info) => Ok(get_file_info.get_file_name.clone()),
                None => Err(Error::new(
                    std::io::ErrorKind::Other,
                    format!("Files service not responding! buy_product_artifact"),
                )),
            }
        }
        None => Err(Error::new(std::io::ErrorKind::Other, "Not Authorized!")),
    }
}
