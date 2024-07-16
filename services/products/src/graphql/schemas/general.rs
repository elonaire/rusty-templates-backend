use async_graphql::{ComplexObject, Enum, InputObject, SimpleObject};
use serde::{Deserialize, Serialize};
use surrealdb::sql::Thing;

#[derive(Clone, Debug, Serialize, Deserialize, SimpleObject, InputObject)]
#[graphql(input_name = "ProductInput")]
#[graphql(complex)]
pub struct Product {
    #[graphql(skip)]
    pub id: Option<Thing>,
    #[graphql(skip)]
    pub owner: Option<Thing>,
    pub name: String,
    pub preview_link: String,
}

#[ComplexObject]
impl Product {
    async fn id(&self) -> String {
        self.id.as_ref().map(|t| &t.id).expect("id").to_raw()
    }
}
