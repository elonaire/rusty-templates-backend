use async_graphql::{ComplexObject, InputObject, SimpleObject};
use serde::{Deserialize, Serialize};
use surrealdb::sql::Thing;

#[derive(Clone, Debug, Serialize, Deserialize, SimpleObject, InputObject)]
#[graphql(input_name = "RatingInput")]
#[graphql(complex)]
pub struct Rating {
    #[graphql(skip)]
    pub id: Option<Thing>,
    pub rating_value: u32,
}

#[ComplexObject]
impl Rating {
    async fn id(&self) -> String {
        self.id.as_ref().map(|t| &t.id).expect("id").to_raw()
    }
}
