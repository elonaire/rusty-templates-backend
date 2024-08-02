use std::sync::Arc;

use crate::graphql::schemas::general::UploadedFile;
use async_graphql::{Context, Error, Object, Result};
use axum::{Extension, http::HeaderMap};
use lib::{integration::{auth::check_auth_from_acl, foreign_key::add_foreign_key_if_not_exists}, utils::{custom_error::ExtendedError, models::{ForeignKey, License, Product, User}}};
use surrealdb::{engine::remote::ws::Client, Surreal};

#[derive(Default)]
pub struct FileMutation;

#[Object]
impl FileMutation {
    pub async fn add_product_artifact(&self, ctx: &Context<'_>, external_product_id: String, external_license_id: String, file_name: String) -> Result<UploadedFile> {
        let db = ctx.data::<Extension<Arc<Surreal<Client>>>>().unwrap();

        if let Some(headers) = ctx.data_opt::<HeaderMap>() {
            let auth_status = check_auth_from_acl(headers.clone()).await?;
            let product_fk_body = ForeignKey {
                table: "product_id".into(),
                column: "product_id".into(),
                foreign_key: external_product_id.clone()
            };

            let license_fk_body = ForeignKey {
                table: "license_id".into(),
                column: "license_id".into(),
                foreign_key: external_license_id.clone()
            };
            let user_fk_body = ForeignKey {
                table: "user_id".into(),
                column: "user_id".into(),
                foreign_key: auth_status.sub.clone()
            };

            let _internal_user = add_foreign_key_if_not_exists::<User>(ctx, user_fk_body).await;
            let internal_product = add_foreign_key_if_not_exists::<Product>(ctx, product_fk_body).await;
            let internal_license = add_foreign_key_if_not_exists::<License>(ctx, license_fk_body).await;

            let mut file_query = db
            .query(
                "
                SELECT * FROM ONLY file WHERE system_filename=$file_name LIMIT 1
                "
            )
            .bind(("file_name", file_name))
            .await
            .map_err(|e| Error::new(e.to_string()))?;

            let file_query_response: Option<UploadedFile> = file_query.take(0)?;

            match file_query_response {
                Some(file_found) => {
                    let mut product_artifact_query = db
                    .query(
                        "
                        BEGIN TRANSACTION;
                        LET $internal_product = type::thing($product_id);
                        LET $internal_license = type::thing($license_id);
                        LET $file = type::thing($file_id);
                        RELATE $internal_product -> product_license_artifact -> $file CONTENT {
                            in: $internal_product,
                            out: $file,
                            license: $internal_license
                        };
                        LET $actual_file = (SELECT * FROM ONLY $file);
                        RETURN $actual_file;
                        COMMIT TRANSACTION;
                        "
                    )
                    .bind(("product_id", format!("product_id:{}", internal_product.unwrap().id.as_ref().map(|t| &t.id).expect("id").to_raw())))
                    .bind(("license_id", format!("license_id:{}", internal_license.unwrap().id.as_ref().map(|t| &t.id).expect("id").to_raw())))
                    .bind(("file_id", format!("file:{}", file_found.id.as_ref().map(|t| &t.id).expect("id").to_raw())))
                    .await
                    .map_err(|e| Error::new(e.to_string()))?;

                    let response: Option<UploadedFile> = product_artifact_query.take(0)?;

                    match response {
                        Some(file) => Ok(file),
                        None => Err(ExtendedError::new("Failed to Add artifact!", Some(500.to_string())).build())
                    }
                },
                None => Err(ExtendedError::new("Invalid file!", Some(400.to_string())).build())
            }
        } else {
            Err(ExtendedError::new("Invalid Request!", Some(400.to_string())).build())
        }
    }

    pub async fn buy_product_artifact(&self, ctx: &Context<'_>, file_name: String) -> Result<UploadedFile> {
        let db = ctx.data::<Extension<Arc<Surreal<Client>>>>().unwrap();

        if let Some(headers) = ctx.data_opt::<HeaderMap>() {
            let auth_status = check_auth_from_acl(headers.clone()).await?;
            buy_product_artifact_util(&ctx, &db, auth_status.sub.clone(), file_name).await
        } else {
            Err(ExtendedError::new("Invalid Request!", Some(400.to_string())).build())
        }
    }

    pub async fn buy_product_artifact_webhook(&self, ctx: &Context<'_>, file_name: String, ext_user_id: String) -> Result<UploadedFile> {
        let db = ctx.data::<Extension<Arc<Surreal<Client>>>>().unwrap();

        if let Some(headers) = ctx.data_opt::<HeaderMap>() {
            let _auth_status = check_auth_from_acl(headers.clone()).await?;
            buy_product_artifact_util(&ctx, &db, ext_user_id.clone(), file_name).await
        } else {
            Err(ExtendedError::new("Invalid Request!", Some(400.to_string())).build())
        }
    }
}

async fn buy_product_artifact_util(ctx: &Context<'_>, db: &Extension<Arc<Surreal<Client>>>, ext_user_id: String, file_name: String) -> Result<UploadedFile> {
    let user_fk_body = ForeignKey {
        table: "user_id".into(),
        column: "user_id".into(),
        foreign_key: ext_user_id
    };

    let internal_user = add_foreign_key_if_not_exists::<User>(ctx, user_fk_body).await;

    let mut file_query = db
    .query(
        "
        BEGIN TRANSACTION;
        LET $internal_user = type::thing($user_id);
        LET $file = (SELECT VALUE id FROM ONLY file WHERE system_filename=$file_name LIMIT 1);
        RELATE $internal_user -> bought_file -> $file CONTENT {
            in: $internal_user,
            out: $file,
        };
        LET $actual_file = (SELECT * FROM ONLY $file);
        RETURN $actual_file;
        COMMIT TRANSACTION;
        "
    )
    .bind(("file_name", file_name))
    .bind(("user_id", format!("user_id:{}", internal_user.unwrap().id.as_ref().map(|t| &t.id).expect("id").to_raw())))
    .await
    .map_err(|e| Error::new(e.to_string()))?;

    let file_query_response: Option<UploadedFile> = file_query.take(0)?;

    match file_query_response {
        Some(file) => Ok(file),
        None => Err(ExtendedError::new("Failed to purchase artifact!", Some(500.to_string())).build())
    }
}
