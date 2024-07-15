use async_graphql::{Context, Error, Object, Result};

pub struct Mutation;

#[Object]
impl Mutation {
    pub async fn square(&self, ctx: &Context<'_>, num: i32) -> Result<i32> {
        Ok(num*num)
    }
}
