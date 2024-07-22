use std::{sync::Arc, env};
use reqwest::{header::HeaderMap as ReqWestHeaderMap, Client as ReqWestClient};

use crate::graphql::schemas::general::ExchangeRatesResponse;
use async_graphql::{Context, Error, Object, Result};
use axum::Extension;
use surrealdb::{engine::remote::ws::Client, Surreal};
use lib::{integration::{auth::check_auth_from_acl, foreign_key::add_foreign_key_if_not_exists}, utils::{custom_error::ExtendedError, models::{ForeignKey, InitializePaymentResponse, User, UserPaymentDetails}}};
use hyper::http::Method;

#[derive(Default)]
pub struct PaymentMutation;

#[Object]
impl PaymentMutation {
    pub async fn initiate_payment(&self, ctx: &Context<'_>, mut user_payment_details: UserPaymentDetails) -> Result<InitializePaymentResponse> {
        let db = ctx.data::<Extension<Arc<Surreal<Client>>>>().unwrap();

        let auth_res_from_acl = check_auth_from_acl(ctx).await?;

        match auth_res_from_acl {
            Some(auth_status) => {
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

                let forex_secret_key = env::var("EXCHANGE_RATES_API_KEY")
                                .expect("Missing the EXCHANGE_RATES_API_KEY environment variable.");

                let forex_response = client
                    .request(
                        Method::GET,
                        format!("https://api.exchangeratesapi.io/v1/latest?access_key={}&base=USD&symbols=KES", forex_secret_key).as_str(),
                    )
                    .send()
                    .await?
                    .json::<ExchangeRatesResponse>()
                    .await?;

                user_payment_details.amount = ((user_payment_details.amount * forex_response.rates.get("KES").unwrap()) * 100 as f64).ceil();

                let paystack_response = client
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

                Ok(paystack_response)
            },
            None => Err(ExtendedError::new("Not Authorized!", Some(403.to_string())).build()),
        }
    }
}
