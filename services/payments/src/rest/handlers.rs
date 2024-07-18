use axum::{
    extract::{Json, Extension},
    http::StatusCode,
    response::IntoResponse,
};
use std::{fs::File, io::Write, sync::Arc};
use surrealdb::{engine::remote::ws::Client, Surreal};

use crate::graphql::schemas::general::UserPaymentDetails;

pub async fn handle_paystack_webhook(Extension(db): Extension<Arc<Surreal<Client>>>, mut body: Json<UserPaymentDetails>) {}
