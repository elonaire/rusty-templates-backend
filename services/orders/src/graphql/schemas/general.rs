use async_graphql::{ComplexObject, InputObject, SimpleObject};
use lib::utils::models::OrderStatus;
use serde::{Deserialize, Serialize};
use surrealdb::sql::Thing;

#[derive(Clone, Debug, Serialize, Deserialize, SimpleObject)]
#[graphql(complex)]
pub struct Order {
    #[graphql(skip)]
    pub id: Option<Thing>,
    pub status: OrderStatus,
}

#[ComplexObject]
impl Order {
    async fn id(&self) -> String {
        self.id.as_ref().map(|t| &t.id).expect("id").to_raw()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, SimpleObject, InputObject)]
#[graphql(input_name = "CartInput")]
#[graphql(complex)]
pub struct Cart {
    #[graphql(skip)]
    pub id: Option<Thing>,
    pub archived: Option<bool>,
    #[graphql(skip)]
    pub owner: Option<Thing>,
    pub total_amount: f64,
    pub updated_at: Option<String>,
}

#[ComplexObject]
impl Cart {
    async fn id(&self) -> String {
        self.id.as_ref().map(|t| &t.id).expect("id").to_raw()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, SimpleObject, InputObject)]
#[graphql(input_name = "CartProductInput")]
#[graphql(complex)]
pub struct CartProduct {
    #[graphql(skip)]
    pub id: Option<Thing>,
    pub quantity: u32,
}

#[ComplexObject]
impl CartProduct {
    async fn id(&self) -> String {
        self.id.as_ref().map(|t| &t.id).expect("id").to_raw()
    }
}
