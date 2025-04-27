use async_graphql::{MergedObject, Object};

use super::{comments::query::CommentsQuery, reviews::query::ReviewsQuery};

#[derive(Default)]
pub struct EmptyQuery;

#[Object]
impl EmptyQuery {
    pub async fn health(&self) -> String {
        "Shared Service is Online!".to_string()
    }
}

#[derive(MergedObject, Default)]
pub struct Query(EmptyQuery, CommentsQuery, ReviewsQuery);
