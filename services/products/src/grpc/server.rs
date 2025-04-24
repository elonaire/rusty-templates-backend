use std::sync::Arc;

use lib::integration::grpc::clients::products_service::{
    products_service_server::ProductsService, GetLicensePriceFactorArgs,
    GetLicensePriceFactorResponse, ProductPrice, ProductSkuArtifact, ProductSkuId, ProductSkuIds,
    ProductSkuPrice, ProductSkuPrices, RetrieveProductSkuArtifactArgs,
};
use surrealdb::{engine::remote::ws::Client, Surreal};
use tonic::{Request, Response, Status};

use crate::utils;

pub struct ProductsServiceImplementation {
    db: Arc<Surreal<Client>>,
}

impl ProductsServiceImplementation {
    pub fn new(db: Arc<Surreal<Client>>) -> Self {
        Self { db }
    }
}

#[async_trait::async_trait]
impl ProductsService for ProductsServiceImplementation {
    async fn get_product_price(
        &self,
        request: Request<ProductSkuId>,
    ) -> Result<Response<ProductPrice>, Status> {
        match utils::products::get_product_price(
            &self.db,
            request.into_inner().product_sku_id.as_str(),
        )
        .await
        {
            Ok(price) => Ok(Response::new(ProductPrice { price })),
            Err(e) => {
                tracing::error!("Couldn't get product price: {}", e);
                Err(Status::internal("Failed"))
            }
        }
    }

    async fn get_product_sku_artifact(
        &self,
        request: Request<RetrieveProductSkuArtifactArgs>,
    ) -> Result<Response<ProductSkuArtifact>, Status> {
        let args = request.into_inner(); // Move once and store the result

        match utils::products::get_product_sku_artifact(&self.db, args.product_sku_id.as_str())
            .await
        {
            Ok(artifact) => Ok(Response::new(ProductSkuArtifact { artifact })),
            Err(_e) => Err(Status::internal("Failed")),
        }
    }

    async fn get_license_price_factor(
        &self,
        request: Request<GetLicensePriceFactorArgs>,
    ) -> Result<Response<GetLicensePriceFactorResponse>, Status> {
        match utils::products::get_license_price_factor(
            &self.db,
            request.into_inner().license_id.as_str(),
        )
        .await
        {
            Ok(price_factor) => Ok(Response::new(GetLicensePriceFactorResponse {
                price_factor,
            })),
            Err(_e) => Err(Status::internal("Failed")),
        }
    }

    async fn retrieve_product_sku_prices(
        &self,
        request: Request<ProductSkuIds>,
    ) -> Result<Response<ProductSkuPrices>, Status> {
        match utils::products::retrieve_product_sku_prices(
            &self.db,
            request.into_inner().product_sku_ids.to_vec(),
        )
        .await
        {
            Ok(prices) => Ok(Response::new(ProductSkuPrices {
                prices: prices
                    .into_iter()
                    .map(|price| price.into())
                    .collect::<Vec<ProductSkuPrice>>(),
            })),
            Err(e) => {
                tracing::error!("Couldn't get product price: {}", e);
                Err(Status::internal("Failed"))
            }
        }
    }
}
