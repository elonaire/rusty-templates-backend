use std::sync::Arc;

use crate::graphql::schemas::general::Product;
use async_graphql::{Context, Error, Object, Result};
use axum::{http::HeaderMap, Extension};
use hyper::header::{AUTHORIZATION, COOKIE};
use lib::{
    integration::{
        foreign_key::add_foreign_key_if_not_exists,
        grpc::clients::files_service::{files_service_client::FilesServiceClient, FileName},
    },
    middleware::auth::graphql::check_auth_from_acl,
    utils::{
        custom_error::ExtendedError,
        grpc::{create_grpc_client, AuthMetaData},
        models::{ForeignKey, UploadedFile, User},
    },
};
use surrealdb::{engine::remote::ws::Client, Surreal};
use tonic::transport::Channel;

#[derive(Default)]
pub struct ProductMutation;

#[Object]
impl ProductMutation {
    pub async fn create_product(&self, ctx: &Context<'_>, product: Product) -> Result<Product> {
        let db = ctx.data::<Extension<Arc<Surreal<Client>>>>().unwrap();

        if let Some(headers) = ctx.data_opt::<HeaderMap>() {
            let auth_status = check_auth_from_acl(headers).await?;

            let foreign_key = ForeignKey {
                table: "user_id".into(),
                column: "user_id".into(),
                foreign_key: auth_status.sub,
            };
            let owner_result =
                add_foreign_key_if_not_exists::<Extension<Arc<Surreal<Client>>>, User>(
                    db,
                    foreign_key,
                )
                .await;

            match owner_result {
                Some(owner) => {
                    let created_product: Product = db
                        .create("product")
                        .content(Product {
                            owner: owner.id,
                            ..product
                        })
                        .await
                        .map_err(|e| Error::new(e.to_string()))?
                        .expect("Error creating product");

                    return Ok(created_product);
                }
                None => Err(ExtendedError::new("Not Authorized!", Some(403.to_string())).build()),
            }
        } else {
            Err(ExtendedError::new("Not Authorized!", Some(403.to_string())).build())
        }
    }

    pub async fn add_product_artifact(
        &self,
        ctx: &Context<'_>,
        product_id: String,
        license_id: String,
        file_name: String,
    ) -> Result<UploadedFile> {
        let db = ctx.data::<Extension<Arc<Surreal<Client>>>>().unwrap();

        if let Some(headers) = ctx.data_opt::<HeaderMap>() {
            let auth_status = check_auth_from_acl(headers).await?;

            let mut request = tonic::Request::new(FileName { file_name });

            let auth_header = headers.get(AUTHORIZATION);
            let cookie_header = headers.get(COOKIE);

            let auth_metadata: AuthMetaData<FileName> = AuthMetaData {
                auth_header,
                cookie_header,
                constructed_grpc_request: Some(&mut request),
            };

            let mut files_grpc_client =
                create_grpc_client::<FileName, FilesServiceClient<Channel>>(
                    "http://[::1]:50053",
                    true,
                    Some(auth_metadata),
                )
                .await
                .map_err(|e| {
                    tracing::error!("Failed to connect to Files service: {}", e);
                    Error::new("Failed to connect to Files service".to_string())
                })?;

            let res = files_grpc_client.get_file_id(request).await?;
            let file_id: String = res.into_inner().file_id;

            let file_fk_body = ForeignKey {
                table: "file_id".into(),
                column: "file_id".into(),
                foreign_key: file_id.clone(),
            };

            let user_fk_body = ForeignKey {
                table: "user_id".into(),
                column: "user_id".into(),
                foreign_key: auth_status.sub.clone(),
            };

            let _internal_user = add_foreign_key_if_not_exists::<
                Extension<Arc<Surreal<Client>>>,
                User,
            >(db, user_fk_body)
            .await;

            let internal_file = add_foreign_key_if_not_exists::<
                Extension<Arc<Surreal<Client>>>,
                UploadedFile,
            >(db, file_fk_body)
            .await;

            let mut product_artifact_query = db
                .query(
                    "
                BEGIN TRANSACTION;
                LET $product = type::thing($product_id);
                LET $license = type::thing($license_id);
                LET $file = type::thing($file_id);

                RELATE $product -> product_license_artifact -> $file CONTENT {
                    license: $license
                };
                LET $internal_file = (SELECT * FROM ONLY $file);
                RETURN $internal_file;
                COMMIT TRANSACTION;
                ",
                )
                .bind(("product_id", format!("product:{}", product_id)))
                .bind(("license_id", format!("license:{}", license_id)))
                .bind((
                    "file_id",
                    format!(
                        "file_id:{}",
                        internal_file
                            .unwrap()
                            .id
                            .as_ref()
                            .map(|t| &t.id)
                            .expect("id")
                            .to_raw()
                    ),
                ))
                .await
                .map_err(|e| Error::new(e.to_string()))?;

            let response: Option<UploadedFile> = product_artifact_query.take(0)?;

            tracing::debug!("Bought artifact: {:?}", response);

            match response {
                Some(file) => Ok(file),
                None => Err(
                    ExtendedError::new("Failed to Add artifact!", Some(500.to_string())).build(),
                ),
            }
        } else {
            Err(ExtendedError::new("Invalid Request!", Some(400.to_string())).build())
        }
    }
}
