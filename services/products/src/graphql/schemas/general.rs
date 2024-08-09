use std::env;

use async_graphql::{ComplexObject, Enum, InputObject, SimpleObject};
// use reqwest::Client as ReqWestClient;
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
    pub slug: Option<String>,
    pub name: String,
    pub price: u64,
    pub preview_link: String,
    pub details_file: String,
    // #[graphql(skip)]
    // pub product_details: String,
    pub screenshot: String,
    pub framework: Option<Framework>,
    pub application_layer: Option<ApplicationLayer>,
    pub ui_framework: Option<UiFramework>,
    pub use_case: Option<UseCase>,
}

#[ComplexObject]
impl Product {
    async fn id(&self) -> String {
        self.id.as_ref().map(|t| &t.id).expect("id").to_raw()
    }

    async fn product_details(&self) -> String {
        let files_service =
            env::var("FILES_SERVICE").expect("Missing the FILES_SERVICE environment variable.");

        let file_url = format!("{}/view/{}", files_service, self.details_file);
        // let client = ReqWestClient::builder()
        //     .danger_accept_invalid_certs(true)
        //     .build()
        //     .unwrap();
        // let logo_image = fs::read("https://imagedelivery.net/fa3SWf5GIAHiTnHQyqU8IQ/5d0feb5f-2b15-4b86-9cf3-1f99372f4600/public")?;
        match reqwest::get(file_url).await {
            Ok(res) => match res.text().await {
                Ok(data) => {
                    let raw_html =
                        markdown::to_html_with_options(data.as_str(), &markdown::Options::gfm());

                    raw_html.unwrap()
                }
                Err(_e) => "".into(),
            },
            Err(_e) => "".into(),
        }
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
    #[serde(rename = "RustyUI")]
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
    #[serde(rename = "EcommerceAdmin")]
    EcommerceAdmin,
    #[graphql(name = "FinanceAdmin")]
    #[serde(rename = "FinanceAdmin")]
    FinanceAdmin,
    #[graphql(name = "IoTAdmin")]
    #[serde(rename = "IoTAdmin")]
    IoTAdmin,
}
