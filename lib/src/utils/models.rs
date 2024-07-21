use serde::{Deserialize, Serialize};
use surrealdb::sql::Thing;
use async_graphql::SimpleObject;

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
