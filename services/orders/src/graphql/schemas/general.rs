use std::env;

use async_graphql::{ComplexObject, Enum, Error, SimpleObject};
use hyper::{
    header::{AUTHORIZATION, COOKIE},
    HeaderMap,
};
use lib::utils::{grpc::create_grpc_client, models::OrderStatus};
use serde::{Deserialize, Serialize};
use surrealdb::sql::Thing;

use lib::integration::grpc::clients::acl_service::{acl_client::AclClient, Empty};
use tonic::transport::Channel;

use crate::utils::cart::calculate_cart_total_amount;

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

#[derive(Clone, Debug, Serialize, Deserialize, SimpleObject)]
#[graphql(complex)]
pub struct Cart {
    #[graphql(skip)]
    pub id: Option<Thing>,
    pub archived: Option<bool>,
    #[graphql(skip)]
    pub owner: Option<Thing>,
    pub products: Vec<CartProduct>,
    pub updated_at: Option<String>,
}

#[ComplexObject]
impl Cart {
    async fn id(&self) -> String {
        self.id.as_ref().map(|t| &t.id).expect("id").to_raw()
    }

    async fn total_amount(&self) -> u64 {
        // Calculate the total amount by summing up the product prices(with license price factor applied), applying tax and discounts.
        // Params: a vector of ProductSkus
        // Send request to the product service to get the product sku prices

        // Internal sign in logic using gRPC
        let request = tonic::Request::new(Empty {});

        let acl_service_grpc = env::var("OAUTH_SERVICE_GRPC")
            .expect("Missing the OAUTH_SERVICE_GRPC environment variable.");

        if let Ok(mut acl_grpc_client) =
            create_grpc_client::<Empty, AclClient<Channel>>(&acl_service_grpc, false, None)
                .await
                .map_err(|e| {
                    tracing::error!("Failed to connect to ACL service: {}", e);
                    Error::new("Failed to connect to ACL service".to_string())
                })
        {
            if let Ok(auth_res) = acl_grpc_client.sign_in_as_service(request).await {
                let mut header_map = HeaderMap::new();
                let internal_jwt = auth_res.into_inner().token;
                header_map.insert(
                    AUTHORIZATION,
                    format!("Bearer {}", &internal_jwt)
                        .as_str()
                        .parse()
                        .unwrap(),
                );
                header_map.insert(
                    COOKIE,
                    format!("oauth_client=;t={}", &internal_jwt)
                        .as_str()
                        .parse()
                        .unwrap(),
                );

                match calculate_cart_total_amount(&header_map, self.products.clone()).await {
                    Ok(sum) => sum,
                    Err(e) => {
                        tracing::error!("Failed to calculate cart total amount: {}", e);
                        0
                    }
                }
            } else {
                0
            }
        } else {
            0
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, SimpleObject)]
// #[graphql(input_name = "CartProductInput")]
// #[graphql(complex)]
pub struct CartProduct {
    // #[graphql(skip)]
    // pub id: Option<Thing>,
    pub product_sku_id: String,
    pub quantity: u32,
}

// #[ComplexObject]
// impl CartProduct {
//     async fn id(&self) -> String {
//         self.id.as_ref().map(|t| &t.id).expect("id").to_raw()
//     }
// }

#[derive(Clone, Debug, Serialize, Deserialize, Enum, Eq, Copy, PartialEq)]
pub enum CartOperation {
    #[graphql(name = "AddProduct")]
    AddProduct,
    #[graphql(name = "RemoveProduct")]
    RemoveProduct,
}
