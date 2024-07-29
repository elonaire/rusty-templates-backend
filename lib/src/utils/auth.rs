use async_graphql::SimpleObject;
use serde::{Deserialize, Serialize};


#[derive(Clone, Debug, Serialize, Deserialize, SimpleObject)]
pub struct AuthStatus {
    #[serde(rename = "isAuth")]
    pub is_auth: bool,
    pub sub: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, SimpleObject)]
pub struct DecodeTokenResponse {
    #[serde(rename = "decodeToken")]
    pub decode_token: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, SimpleObject)]
pub struct CheckAuthResponse {
    #[serde(rename = "checkAuth")]
    pub check_auth: AuthStatus,
}
