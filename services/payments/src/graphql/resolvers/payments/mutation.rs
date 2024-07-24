use std::env;
use reqwest::{header::HeaderMap as ReqWestHeaderMap, Client as ReqWestClient};

use crate::graphql::schemas::general::ExchangeRatesResponse;
use async_graphql::{Context, Object, Result, Error};
use lib::{integration::auth::check_auth_from_acl, utils::{models::{InitializePaymentResponse, UserPaymentDetails}, custom_error::ExtendedError}};
use hyper::http::Method;
use axum::http::HeaderMap;

#[derive(Default)]
pub struct PaymentMutation;

#[Object]
impl PaymentMutation {
    pub async fn initiate_payment(&self, ctx: &Context<'_>, mut user_payment_details: UserPaymentDetails) -> Result<InitializePaymentResponse> {
        if let Some(headers) = ctx.data_opt::<HeaderMap>() {
            let _auth_status = check_auth_from_acl(headers.clone()).await?;

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
                .await.map_err(|e| {
                    println!("Error sending: {:?}", e);
                    Error::new(e.to_string())
                })?
                .json::<ExchangeRatesResponse>()
                .await.map_err(|e| {
                    println!("Error deserializing: {:?}", e);
                    Error::new(e.to_string())
                })?;

            println!("Passes forex_response! {:?}", forex_response);
            let conversion_rate = forex_response.rates.get("KES").unwrap();

            user_payment_details.amount = (user_payment_details.amount as f64 * *conversion_rate * 100.0) as u64;

            println!("user_payment_details {:?}", user_payment_details);

            let paystack_response = client
                .request(
                    Method::POST,
                    "https://api.paystack.co/transaction/initialize",
                )
                .headers(req_headers)
                .json::<UserPaymentDetails>(&user_payment_details)
                .send()
                .await.map_err(|e| {
                    println!("sending error: {:?}", e);
                    Error::new(e.to_string())
                })?
                .json::<InitializePaymentResponse>()
                .await.map_err(|e| {
                    println!("Decoding error: {:?}", e);
                    Error::new(e.to_string())
                })?;

            println!("Passes paystack_response! {:?}", paystack_response);

            Ok(paystack_response)
        } else {
            Err(ExtendedError::new("Not Authorized!", Some(403.to_string())).build())
        }
    }
}
