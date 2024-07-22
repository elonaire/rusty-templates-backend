use serde::{Deserialize, Serialize};
use surrealdb::sql::Thing;
use async_graphql::{SimpleObject, InputObject};

#[derive(Clone, Debug, Serialize, Deserialize, SimpleObject)]
pub struct User {
    #[graphql(skip)]
    pub id: Option<Thing>,
    pub user_id: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ForeignKey {
    pub table: String,
    pub column: String,
    pub foreign_key: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, SimpleObject)]
pub struct Product {
    #[graphql(skip)]
    pub id: Option<Thing>,
    pub product_id: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, SimpleObject)]
pub struct AuthStatus {
    pub is_auth: bool,
    pub sub: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetUserVar {
    pub id: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct GetUserResponse {
    #[serde(rename = "getUserEmail")]
    pub get_user_email: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InitiatePaymentVar {
    #[serde(rename = "userPaymentDetails")]
    pub user_payment_details: UserPaymentDetails
}

#[derive(Clone, Debug, Serialize, Deserialize, SimpleObject, InputObject)]
#[graphql(input_name = "UserPaymentDetailsInput")]
pub struct UserPaymentDetails {
    pub email: String,
    pub amount: f64,
    pub currency: Option<String>,
    pub metadata: Option<PaymentDetailsMetaData>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct InitPaymentGraphQLResponse {
    #[serde(rename = "InitiatePayment")]
    pub initiate_payment: InitializePaymentResponse,
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
