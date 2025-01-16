use async_graphql::{ComplexObject, InputObject, SimpleObject};
use serde::{Deserialize, Serialize};
use surrealdb::sql::Thing;

#[derive(Clone, Debug, Serialize, Deserialize, SimpleObject, InputObject)]
#[graphql(input_name = "CommentInput")]
#[graphql(complex)]
pub struct Comment {
    #[graphql(skip)]
    pub id: Option<Thing>,
    pub content: String,
}

#[ComplexObject]
impl Comment {
    async fn id(&self) -> String {
        self.id.as_ref().map(|t| &t.id).expect("id").to_raw()
    }
}
