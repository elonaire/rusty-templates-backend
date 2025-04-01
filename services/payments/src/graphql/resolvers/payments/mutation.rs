// use crate::graphql::schemas::general::ExchangeRatesResponse;
use async_graphql::{Context, Object, Result};
use axum::http::HeaderMap;
use lib::{
    middleware::auth::graphql::check_auth_from_acl,
    utils::{
        custom_error::ExtendedError,
        models::{InitializePaymentResponse, UserPaymentDetails},
    },
};

use crate::utils::payments::initiate_payment_integration;

#[derive(Default)]
pub struct PaymentMutation;

#[Object]
impl PaymentMutation {
    pub async fn initiate_payment(
        &self,
        ctx: &Context<'_>,
        mut user_payment_details: UserPaymentDetails,
    ) -> Result<InitializePaymentResponse> {
        if let Some(headers) = ctx.data_opt::<HeaderMap>() {
            let _auth_status = check_auth_from_acl(headers).await?;

            let payment_req = initiate_payment_integration(&mut user_payment_details).await?;

            Ok(payment_req)
        } else {
            Err(ExtendedError::new("Not Authorized!", Some(403.to_string())).build())
        }
    }
}
