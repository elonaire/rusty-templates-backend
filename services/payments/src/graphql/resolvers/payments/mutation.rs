use std::{sync::Arc, env};
use reqwest::{header::HeaderMap as ReqWestHeaderMap, Client as ReqWestClient};

use crate::graphql::schemas::general::{InitializePaymentResponse, UserPaymentDetails};
use async_graphql::{Context, Error, Object, Result};
use axum::Extension;
use surrealdb::{engine::remote::ws::Client, Surreal};
use lib::{middleware::{auth::check_auth_from_acl, foreign_key::add_foreign_key_if_not_exists}, utils::{models::{ForeignKey, User}, custom_error::ExtendedError}};
use hyper::http::Method;

#[derive(Default)]
pub struct PaymentMutation;

#[Object]
impl PaymentMutation {
    pub async fn initiate_payment(&self, user_payment_details: UserPaymentDetails) -> Result<InitializePaymentResponse> {
        let client = ReqWestClient::new();
        let paystack_secret = env::var("PAYSTACK_SECRET")
                        .expect("Missing the PAYSTACK_SECRET environment variable.");

        let mut req_headers = ReqWestHeaderMap::new();
        req_headers
            .insert("Authorization", format!("Bearer {}", paystack_secret).as_str().parse().unwrap());

        req_headers.append(
            "Cache-Control",
            "no-cache".parse().unwrap(),
        );

        println!("req_headers: {:?}", req_headers);

        let response = client
            .request(
                Method::POST,
                "https://api.paystack.co/transaction/initialize",
            )
            .headers(req_headers)
            .json::<UserPaymentDetails>(&user_payment_details)
            .send()
            .await?
            .json::<InitializePaymentResponse>()
            .await?;

        println!("response: {:?}", response);

        Ok(response)
    }
}
