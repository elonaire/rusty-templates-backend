use std::sync::Arc;

use products_service::{
    products_service_server::ProductsService, GetLicensePriceFactorArgs,
    GetLicensePriceFactorResponse, ProductArtifact, ProductId, ProductPrice,
    RetrieveProductArtifactArgs,
};
use surrealdb::{engine::remote::ws::Client, Surreal};
use tonic::{Request, Response, Status};

use crate::utils;

pub mod products_service {
    tonic::include_proto!("products");
}

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
        request: Request<ProductId>,
    ) -> Result<Response<ProductPrice>, Status> {
        match utils::products::get_product_price(&self.db, request.into_inner().product_id.as_str())
            .await
        {
            Ok(price) => Ok(Response::new(ProductPrice { price })),
            Err(_e) => Err(Status::internal("Failed")),
        }
    }

    async fn get_product_artifact(
        &self,
        request: Request<RetrieveProductArtifactArgs>,
    ) -> Result<Response<ProductArtifact>, Status> {
        let args = request.into_inner(); // Move once and store the result

        match utils::products::get_product_artifact(
            &self.db,
            args.product_id.as_str(),
            args.license_id.as_str(),
        )
        .await
        {
            Ok(artifact) => Ok(Response::new(ProductArtifact { artifact })),
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
}
