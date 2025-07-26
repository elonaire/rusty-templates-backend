use std::{env, sync::Arc};

use crate::graphql::schemas::general::{
    License, Product, ProductInput, ProductSku, ProductSkuInput, UpdateLicenseInput,
};
use async_graphql::{Context, Error, Object, Result};
use axum::{http::HeaderMap, Extension};
use hyper::{
    header::{AUTHORIZATION, COOKIE},
    StatusCode,
};
use lib::{
    integration::{
        foreign_key::add_foreign_key_if_not_exists,
        grpc::clients::files_service::{files_service_client::FilesServiceClient, FileId},
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
    /// Create New Product
    pub async fn create_product(
        &self,
        ctx: &Context<'_>,
        mut product: ProductInput,
    ) -> Result<Product> {
        let db = ctx.data::<Extension<Arc<Surreal<Client>>>>().map_err(|e| {
            tracing::error!("Error extracting Surreal Client: {:?}", e);
            ExtendedError::new(
                "Server Error",
                Some(StatusCode::INTERNAL_SERVER_ERROR.as_u16()),
            )
            .build()
        })?;

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
                    product.owner = owner.id;

                    let mut create_product_query = db
                        .query(
                            "
                            BEGIN TRANSACTION;
                            LET $product = CREATE product CONTENT $product_input;
                            RETURN $product;
                            COMMIT TRANSACTION;
                            ",
                        )
                        .bind(("product_input", product))
                        .await
                        .map_err(|e| {
                            tracing::error!("DB Query Error: {}", e);
                            ExtendedError::new(
                                "Product not created",
                                Some(StatusCode::BAD_REQUEST.as_u16()),
                            )
                            .build()
                        })?;

                    let created_product: Option<Product> =
                        create_product_query.take(0).map_err(|e| {
                            tracing::error!("Deserialization Error: {}", e);
                            ExtendedError::new(
                                "Product not created",
                                Some(StatusCode::BAD_REQUEST.as_u16()),
                            )
                            .build()
                        })?;

                    match created_product {
                        Some(product) => Ok(product),
                        None => {
                            tracing::error!("None(Product) was created");
                            Err(ExtendedError::new(
                                "Product not created",
                                Some(StatusCode::BAD_REQUEST.as_u16()),
                            )
                            .build())
                        }
                    }
                }
                None => Err(ExtendedError::new(
                    "Not Authorized!",
                    Some(StatusCode::UNAUTHORIZED.as_u16()),
                )
                .build()),
            }
        } else {
            Err(
                ExtendedError::new("Not Authorized!", Some(StatusCode::UNAUTHORIZED.as_u16()))
                    .build(),
            )
        }
    }

    /// Create New Product SKU
    pub async fn create_product_sku(
        &self,
        ctx: &Context<'_>,
        product_sku_input: ProductSkuInput,
    ) -> Result<ProductSku> {
        let db = ctx.data::<Extension<Arc<Surreal<Client>>>>().map_err(|e| {
            tracing::error!("Error extracting Surreal Client: {:?}", e);
            ExtendedError::new(
                "Server Error",
                Some(StatusCode::INTERNAL_SERVER_ERROR.as_u16()),
            )
            .build()
        })?;

        if let Some(headers) = ctx.data_opt::<HeaderMap>() {
            let auth_status = check_auth_from_acl(headers).await?;

            let mut request = tonic::Request::new(FileId {
                file_id: product_sku_input.file_id.clone(),
            });

            let auth_header = headers.get(AUTHORIZATION);
            let cookie_header = headers.get(COOKIE);

            let auth_metadata: AuthMetaData<FileId> = AuthMetaData {
                auth_header,
                cookie_header,
                constructed_grpc_request: Some(&mut request),
            };

            let files_service_grpc = env::var("FILES_SERVICE_GRPC").map_err(|e| {
                tracing::error!(
                    "Missing the FILES_SERVICE_GRPC environment variable.: {}",
                    e
                );
                ExtendedError::new(
                    "Server Error",
                    Some(StatusCode::INTERNAL_SERVER_ERROR.as_u16()),
                )
                .build()
            })?;

            let mut files_grpc_client = create_grpc_client::<FileId, FilesServiceClient<Channel>>(
                &files_service_grpc,
                true,
                Some(auth_metadata),
            )
            .await
            .map_err(|e| {
                tracing::error!("Failed to connect to Files service: {}", e);
                ExtendedError::new(
                    "Service Unavailable",
                    Some(StatusCode::SERVICE_UNAVAILABLE.as_u16()),
                )
                .build()
            })?;

            let _res = files_grpc_client.get_file_name(request).await?;
            // let _file_name: String = res.into_inner().file_id;

            let file_fk_body = ForeignKey {
                table: "file_id".into(),
                column: "file_id".into(),
                foreign_key: product_sku_input.file_id.clone(),
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

            if internal_file.is_none() {
                tracing::error!("Invalid internal file.");
                return Err(ExtendedError::new(
                    "Bad Request!",
                    Some(StatusCode::BAD_REQUEST.as_u16()),
                )
                .build());
            }

            let mut product_sku_query = db
                .query(
                    "
                    BEGIN TRANSACTION;
                    LET $artifact = type::thing('file_id', $file_id);
                    LET $license = type::thing('license', $license_id);
                    LET $product = type::thing('product', $product_id);

                    LET $product_sku = RELATE $product -> product_sku -> $product CONTENT {
                    artifact: $artifact,
                    license: $license
                    } RETURN AFTER;
                    LET $product_sku_id = SELECT VALUE id FROM ONLY $product_sku LIMIT 1;
                    LET $product_sku_full = SELECT *, artifact[*], license[*] FROM ONLY $product_sku_id LIMIT 1;
                    RETURN $product_sku_full;
                    COMMIT TRANSACTION;
                    ",
                )
                .bind((
                    "file_id",
                    internal_file
                        .unwrap()
                        .id
                        .as_ref()
                        .map(|t| &t.id)
                        .expect("id")
                        .to_raw(),
                ))
                .bind(("license_id", product_sku_input.license_id))
                .bind(("product_id", product_sku_input.product_id))
                .await
                .map_err(|e| {
                    tracing::error!("DB Query Error: {}", e);
                    ExtendedError::new("Product SKU not created", Some(StatusCode::BAD_REQUEST.as_u16())).build()
                })?;

            let response: Option<ProductSku> = product_sku_query.take(0).map_err(|e| {
                tracing::error!("Deserialization Error: {}", e);
                ExtendedError::new(
                    "Product SKU not created",
                    Some(StatusCode::BAD_REQUEST.as_u16()),
                )
                .build()
            })?;

            tracing::debug!("ProductSku: {:?}", response);

            match response {
                Some(product_sku) => Ok(product_sku),
                None => Err(ExtendedError::new(
                    "Failed to Add artifact!",
                    Some(StatusCode::INTERNAL_SERVER_ERROR.as_u16()),
                )
                .build()),
            }
        } else {
            Err(
                ExtendedError::new("Invalid Request!", Some(StatusCode::BAD_REQUEST.as_u16()))
                    .build(),
            )
        }
    }

    /// Update a License
    pub async fn update_license(
        &self,
        ctx: &Context<'_>,
        license_updates: UpdateLicenseInput,
        license_id: String,
    ) -> Result<License> {
        let db = ctx.data::<Extension<Arc<Surreal<Client>>>>().map_err(|e| {
            tracing::error!("Error extracting Surreal Client: {:?}", e);
            ExtendedError::new(
                "Server Error",
                Some(StatusCode::INTERNAL_SERVER_ERROR.as_u16()),
            )
            .build()
        })?;

        if let Some(headers) = ctx.data_opt::<HeaderMap>() {
            let _auth_res_from_acl = check_auth_from_acl(headers).await?;

            let response: Option<License> = db
                .update(("license", license_id))
                .merge(license_updates)
                .await
                .map_err(|e| {
                    tracing::debug!("DB Query Error: {}", e);
                    Error::new("Internal Server Error")
                })?;

            match response {
                Some(license) => Ok(license),
                None => Err(ExtendedError::new(
                    "License not found",
                    Some(StatusCode::NOT_FOUND.as_u16()),
                )
                .build()),
            }
        } else {
            Err(
                ExtendedError::new("Invalid Request!", Some(StatusCode::BAD_REQUEST.as_u16()))
                    .build(),
            )
        }
    }
}
