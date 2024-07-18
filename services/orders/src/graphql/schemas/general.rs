use async_graphql::{ComplexObject, Enum, InputObject, SimpleObject};
use serde::{Deserialize, Serialize};
use surrealdb::sql::Thing;

#[derive(Clone, Debug, Serialize, Deserialize, SimpleObject, InputObject)]
#[graphql(input_name = "OrderInput")]
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


#[derive(Clone, Debug, Serialize, Deserialize, Enum, Copy, Eq, PartialEq)]
pub enum OrderStatus {
    #[graphql(name = "Pending")]
    Pending,
    #[graphql(name = "Confirmed")]
    Confirmed,
    #[graphql(name = "Ready")]
    Ready,
    #[graphql(name = "Completed")]
    Completed,
    #[graphql(name = "Failed")]
    Failed,
    #[graphql(name = "Refunded")]
    Refunded,
    #[graphql(name = "OnHold")]
    OnHold,
}

#[derive(Clone, Debug, Serialize, Deserialize, SimpleObject, InputObject)]
#[graphql(input_name = "CartInput")]
#[graphql(complex)]
pub struct Cart {
    #[graphql(skip)]
    pub id: Option<Thing>,
    pub archived: Option<bool>,
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
