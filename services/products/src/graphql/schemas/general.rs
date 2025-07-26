use std::env;

use async_graphql::{ComplexObject, Enum, InputObject, SimpleObject};
use lib::utils::models::UploadedFile;
// use reqwest::Client as ReqWestClient;
use serde::{Deserialize, Serialize};
use surrealdb::sql::Thing;

#[derive(Clone, Debug, Serialize, Deserialize, SimpleObject, InputObject)]
#[graphql(input_name = "ProductInput")]
pub struct ProductInput {
    #[graphql(skip)]
    pub id: Option<Thing>,
    #[graphql(skip)]
    pub owner: Option<Thing>,
    #[graphql(skip)]
    pub slug: String,
    pub name: String,
    pub price: u64,
    pub preview_link: String,
    pub details_file: String,
    pub screenshot: String,
    pub framework: Option<Framework>,
    pub application_layer: Option<ApplicationLayer>,
    pub ui_framework: Option<UiFramework>,
    pub use_case: Option<UseCase>,
}

#[derive(Clone, Debug, Serialize, Deserialize, SimpleObject)]
#[graphql(complex)]
pub struct Product {
    #[graphql(skip)]
    pub id: Option<Thing>,
    #[graphql(skip)]
    pub owner: Option<Thing>,
    pub slug: String,
    pub name: String,
    pub price: u64,
    pub preview_link: String,
    pub details_file: String,
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

    async fn owner(&self) -> String {
        self.owner.as_ref().map(|t| &t.id).expect("owner").to_raw()
    }

    async fn product_details(&self) -> Option<String> {
        let files_service = env::var("FILES_SERVICE")
            .map_err(|e| {
                tracing::error!("Missing the FILES_SERVICE environment variable.: {}", e);
            })
            .ok()?;

        let file_url = format!("{}/view/{}", files_service, self.details_file);

        match reqwest::get(file_url).await {
            Ok(res) => match res.text().await {
                Ok(data) => {
                    let raw_html =
                        markdown::to_html_with_options(data.as_str(), &markdown::Options::gfm());

                    Some(
                        raw_html
                            .map_err(|e| {
                                tracing::error!("Failed to convert MD to HTML: {}", e);
                            })
                            .ok()?,
                    )
                }
                Err(_e) => None,
            },
            Err(_e) => None,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Enum, Copy, Eq, PartialEq)]
pub enum Framework {
    #[graphql(name = "Yew")]
    Yew,
    #[graphql(name = "Dioxus")]
    Dioxus,
    #[graphql(name = "Leptos")]
    Leptos,
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

#[derive(Clone, Debug, Serialize, Deserialize, SimpleObject, InputObject)]
#[graphql(input_name = "ProductSkuInput")]
pub struct ProductSkuInput {
    pub product_id: String,
    pub license_id: String,
    pub file_id: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, SimpleObject)]
#[graphql(complex)]
pub struct ProductSku {
    #[graphql(skip)]
    pub id: Option<Thing>,
    pub license: License,
    pub artifact: UploadedFile,
}

#[ComplexObject]
impl ProductSku {
    async fn id(&self) -> String {
        self.id.as_ref().map(|t| &t.id).expect("id").to_raw()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, SimpleObject, InputObject)]
#[graphql(input_name = "UpdateLicenseInput")]
pub struct UpdateLicenseInput {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub price_factor: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub short_description: Option<String>,
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
