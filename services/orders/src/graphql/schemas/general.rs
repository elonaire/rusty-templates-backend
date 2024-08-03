use async_graphql::{ComplexObject, Enum, InputObject, SimpleObject};
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
    pub total_amount: u64,
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
    #[graphql(skip)]
    pub license: Option<Thing>,
    pub quantity: u32,
    pub ext_product_id: String,
    pub artifact: String,
}

#[ComplexObject]
impl CartProduct {
    async fn id(&self) -> String {
        self.id.as_ref().map(|t| &t.id).expect("id").to_raw()
    }

    async fn license(&self) -> String {
        self.license.as_ref().map(|t| &t.id).expect("license").to_raw()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Enum, Eq, Copy, PartialEq)]
pub enum CartOperation {
    #[graphql(name = "AddProduct")]
    AddProduct,
    #[graphql(name = "RemoveProduct")]
    RemoveProduct
}

#[derive(Clone, Debug, Serialize, Deserialize, SimpleObject)]
#[graphql(complex)]
pub struct License {
    #[graphql(skip)]
    pub id: Option<Thing>,
    pub name: String,
    pub price_factor: u64,
    pub short_description: String,
}

#[ComplexObject]
impl License {
    async fn id(&self) -> String {
        self.id.as_ref().map(|t| &t.id).expect("id").to_raw()
    }
}
