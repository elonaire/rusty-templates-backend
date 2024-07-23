use axum::{
    extract::{Extension, Json},
    http::StatusCode,
    response::IntoResponse,
};
use lib::utils::models::UserPaymentDetails;
use std::{fs::File, io::Write, sync::Arc};
use surrealdb::{engine::remote::ws::Client, Surreal};

use crate::graphql::schemas::paystack::ChargeEvent;

pub async fn handle_paystack_webhook(Extension(db): Extension<Arc<Surreal<Client>>>, Json(body): Json<ChargeEvent>) -> Json<bool> {
    println!("body: {:?}", body);
    if body.event == "charge.success".to_string() {
        println!("charge.success body: {:?}", body);
    }

    Json(true)
}
