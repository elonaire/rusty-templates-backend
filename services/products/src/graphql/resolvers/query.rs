use async_graphql::{Context, Error, Object, Result};

pub struct Query;

#[Object]
impl Query {
    pub async fn square(&self, ctx: &Context<'_>, num: i32) -> Result<i32> {
        Ok(num*num)
    }
}
