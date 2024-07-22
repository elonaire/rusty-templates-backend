use axum::{
    extract::{Json, Extension},
    http::StatusCode,
    response::IntoResponse,
};
use lib::utils::models::UserPaymentDetails;
use std::{fs::File, io::Write, sync::Arc};
use surrealdb::{engine::remote::ws::Client, Surreal};

pub async fn handle_paystack_webhook(Extension(db): Extension<Arc<Surreal<Client>>>, mut body: Json<UserPaymentDetails>) {}
