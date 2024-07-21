use async_graphql::{InputObject, SimpleObject};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, SimpleObject, InputObject)]
#[graphql(input_name = "UserPaymentDetailsInput")]
pub struct UserPaymentDetails {
    pub email: String,
    pub amount: f64,
    pub currency: Option<String>,
    pub metadata: Option<PaymentDetailsMetaData>,
}

#[derive(Clone, Debug, Serialize, Deserialize, SimpleObject)]
pub struct InitializePaymentResponse {
    pub status: bool,
    pub message: String,
    pub data: InitializePaymentResponseData,
}

#[derive(Clone, Debug, Serialize, Deserialize, SimpleObject)]
pub struct InitializePaymentResponseData {
    pub authorization_url: String,
    pub access_code: String,
    pub reference: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, SimpleObject, InputObject)]
pub struct PaymentDetailsMetaData {
    pub cart_id: Option<String>,
}
