use serde::{Deserialize, Serialize};
use surrealdb::sql::Thing;
use async_graphql::{Enum, InputObject, SimpleObject};

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
    #[serde(rename = "isAuth")]
    pub is_auth: bool,
    pub sub: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetUserVar {
    pub id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetProductPriceVar {
    #[serde(rename = "productId")]
    pub product_id: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct GetUserResponse {
    #[serde(rename = "getUserEmail")]
    pub get_user_email: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct GetProductPriceResponse {
    #[serde(rename = "getProductPrice")]
    pub get_product_price: u64,
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
    pub amount: u64,
    // pub currency: Option<String>,
    pub reference: String,
    // pub metadata: Option<PaymentDetailsMetaData>,
}

#[derive(Debug, Deserialize, Serialize, SimpleObject)]
pub struct InitPaymentGraphQLResponse {
    #[serde(rename = "initiatePayment")]
    pub initiate_payment: InitializePaymentGraphQLResponse,
}

#[derive(Clone, Debug, Serialize, Deserialize, SimpleObject)]
pub struct InitializePaymentResponse {
    pub status: bool,
    pub message: String,
    pub data: InitializePaymentResponseData,
}

// For GraphQL because of the camel-case convention
#[derive(Clone, Debug, Serialize, Deserialize, SimpleObject)]
pub struct InitializePaymentGraphQLResponse {
    pub status: bool,
    pub message: String,
    pub data: InitializePaymentGraphQLResponseData,
}

#[derive(Clone, Debug, Serialize, Deserialize, SimpleObject)]
pub struct InitializePaymentResponseData {
    #[serde(rename = "authorization_url")]
    pub authorization_url: String,
    #[serde(rename = "access_code")]
    pub access_code: String,
    pub reference: String,
}

// For GraphQL because of the camel-case convention
#[derive(Clone, Debug, Serialize, Deserialize, SimpleObject)]
pub struct InitializePaymentGraphQLResponseData {
    #[serde(rename = "authorizationUrl")]
    pub authorization_url: String,
    #[serde(rename = "accessCode")]
    pub access_code: String,
    pub reference: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, SimpleObject, InputObject)]
pub struct PaymentDetailsMetaData {
    #[serde(rename = "cartId")]
    pub cart_id: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Enum, Copy, Eq, PartialEq)]
pub enum OrderStatus {
    #[graphql(name = "Pending")]
    Pending,
    #[graphql(name = "Confirmed")]
    Confirmed,
    #[graphql(name = "Ready")]
    Ready,
    #[graphql(name = "Completed")]
    Completed,
    #[graphql(name = "Failed")]
    Failed,
    #[graphql(name = "Refunded")]
    Refunded,
    #[graphql(name = "OnHold")]
    OnHold,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateOrderVar {
    #[serde(rename = "orderId")]
    pub order_id: String,
    pub status: OrderStatus,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateOrderResponse {
    #[serde(rename = "updateOrder")]
    pub update_order: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, SimpleObject, InputObject)]
#[graphql(input_name = "EmailInput")]
pub struct Email {
    pub recipient: EmailUser,
    pub subject: String,
    pub title: String,
    pub body: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, SimpleObject, InputObject)]
pub struct EmailUser {
    #[serde(rename = "fullName")]
    pub full_name: Option<String>,
    #[serde(rename = "emailAddress")]
    pub email_address: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SendEmailVar {
    pub email: Email,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SendEmailResponse {
    #[serde(rename = "sendEmail")]
    pub send_email: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, SimpleObject, InputObject)]
#[graphql(input_name = "UserLoginsInput")]
pub struct UserLogins {
    #[serde(rename = "userName")]
    pub user_name: Option<String>,
    #[graphql(secret)]
    pub password: Option<String>,
    // pub oauth_client: Option<OAuthClientName>,
}

#[derive(Clone, Debug, Serialize, Deserialize, SimpleObject)]
pub struct AuthDetails {
    // pub url: Option<String>,
    pub token: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SignInResponse {
    #[serde(rename = "signIn")]
    pub sign_in: AuthDetails,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserLoginsVar {
    #[serde(rename = "rawUserDetails")]
    pub raw_user_details: UserLogins
}

#[derive(Clone, Debug, Serialize, Deserialize, SimpleObject)]
pub struct License {
    #[graphql(skip)]
    pub id: Option<Thing>,
    pub license_id: String,
}
