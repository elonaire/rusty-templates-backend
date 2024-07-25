use async_graphql::{MergedObject, Object};

use super::products::query::ProductQuery;

#[derive(Default)]
pub struct EmptyQuery;

#[Object]
impl EmptyQuery {
    pub async fn health(&self) -> String {
        "Products Service is Online!".to_string()
    }
}

#[derive(MergedObject, Default)]
pub struct Query(EmptyQuery, ProductQuery);
