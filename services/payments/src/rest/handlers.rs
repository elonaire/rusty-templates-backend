use axum::{
    extract::{Json, Extension},
    http::StatusCode,
    response::IntoResponse,
};
use lib::utils::models::UserPaymentDetails;
use std::{fs::File, io::Write, sync::Arc};
use surrealdb::{engine::remote::ws::Client, Surreal};

use crate::graphql::schemas::paystack::ChargeEvent;

pub async fn handle_paystack_webhook(Extension(db): Extension<Arc<Surreal<Client>>>, mut body: Json<ChargeEvent>) {
    if body.event == "charge.success".to_string() {
        println!("body: {:?}", body);
    }
}
