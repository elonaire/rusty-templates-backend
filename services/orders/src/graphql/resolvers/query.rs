use async_graphql::{MergedObject, Object};

use super::{cart::query::CartQuery, orders::query::OrderQuery};

#[derive(Default)]
pub struct EmptyQuery;

#[Object]
impl EmptyQuery {
    pub async fn health(&self) -> String {
        "Orders Service is Online!".to_string()
    }
}

#[derive(MergedObject, Default)]
pub struct Query(EmptyQuery, CartQuery, OrderQuery);
