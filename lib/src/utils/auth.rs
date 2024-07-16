use std::sync::Arc;

use async_graphql::{Context, Result, SimpleObject};
use axum::{http::HeaderValue, Extension};
use jwt_simple::prelude::*;
use serde::{Deserialize, Serialize};
use surrealdb::{engine::remote::ws::Client, Surreal};
use surrealdb::sql::Thing;

use super::custom_error::ExtendedError;

#[derive(Clone, Debug, Serialize, Deserialize, SimpleObject)]
pub struct AuthStatus {
    pub is_auth: bool,
    pub sub: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, SimpleObject)]
pub struct DecodeTokenResponse {
    #[serde(rename = "decodeToken")]
    pub decode_token: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, SimpleObject)]
pub struct User {
    #[graphql(skip)]
    pub id: Option<Thing>,
    pub user_id: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, SimpleObject)]
pub struct ForeignKey{
    #[graphql(skip)]
    pub id: Option<Thing>,
    pub table: String,
    pub column: String,
    pub foreign_key: String,
}
