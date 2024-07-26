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
    pub price: f64,
    pub preview_link: String,
    pub screenshot: String,
    pub framework: Framework,
    pub application_layer: ApplicationLayer,
    pub ui_framework: Option<UiFramework>,
    pub use_case: UseCase,
}

#[ComplexObject]
impl Product {
    async fn id(&self) -> String {
        self.id.as_ref().map(|t| &t.id).expect("id").to_raw()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Enum, Copy, Eq, PartialEq)]
pub enum Framework {
    #[graphql(name = "Yew")]
    Yew,
    #[graphql(name = "Dioxus")]
    Dioxus,
    #[graphql(name = "Axum")]
    Axum,
    #[graphql(name = "Rocket")]
    Rocket,
    #[graphql(name = "Iced")]
    Iced,
    #[graphql(name = "Tauri")]
    Tauri,
    #[graphql(name = "Actix")]
    Actix,
    #[graphql(name = "Warp")]
    Warp,
    #[graphql(name = "Rouille")]
    Rouille,
    #[graphql(name = "Thruster")]
    Thruster,
}

#[derive(Clone, Debug, Serialize, Deserialize, Enum, Copy, Eq, PartialEq)]
pub enum ApplicationLayer {
    #[graphql(name = "Frontend")]
    Frontend,
    #[graphql(name = "Backend")]
    Backend,
}

#[derive(Clone, Debug, Serialize, Deserialize, Enum, Copy, Eq, PartialEq)]
pub enum UiFramework {
    #[graphql(name = "RustyUI")]
    #[serde(rename = "Rusty UI")]
    RustyUI,
}

#[derive(Clone, Debug, Serialize, Deserialize, Enum, Copy, Eq, PartialEq)]
pub enum UseCase {
    #[graphql(name = "Dashboard")]
    Dashboard,
    #[graphql(name = "Ecommerce")]
    Ecommerce,
    #[graphql(name = "Admin")]
    Admin,
    #[graphql(name = "EcommerceAdmin")]
    #[serde(rename = "Ecommerce Admin")]
    EcommerceAdmin,
    #[graphql(name = "FinanceAdmin")]
    #[serde(rename = "Finance Admin")]
    FinanceAdmin,
    #[graphql(name = "IoTAdmin")]
    #[serde(rename = "IoT Admin")]
    IoTAdmin,
}
