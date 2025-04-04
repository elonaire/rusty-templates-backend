use std::sync::Arc;

use orders_service::{
    orders_service_server::OrdersService, ArtifactsPurchaseDetails, GetAllArtifactsForOrderPayload,
    UpdateOrderPayload, UpdateOrderResponse,
};
use surrealdb::{engine::remote::ws::Client, Surreal};
use tonic::{Request, Response, Status};

use crate::utils;

pub mod orders_service {
    tonic::include_proto!("orders");
}

pub struct OrdersServiceImplementation {
    db: Arc<Surreal<Client>>,
}

impl OrdersServiceImplementation {
    pub fn new(db: Arc<Surreal<Client>>) -> Self {
        Self { db }
    }
}

#[async_trait::async_trait]
impl OrdersService for OrdersServiceImplementation {
    async fn update_order(
        &self,
        request: Request<UpdateOrderPayload>,
    ) -> Result<Response<UpdateOrderResponse>, Status> {
        let req_clone = request.extensions().clone();
        let current_user = req_clone.get::<String>().unwrap();
        let payload = request.into_inner();
        let status = payload
            .status
            .try_into()
            .map_err(|_| Status::invalid_argument("Invalid status"))?;

        tracing::debug!("status: {:?}", status);

        match utils::orders::update_order(&self.db, payload.order_id.as_str(), status).await {
            Ok(status_str) => Ok(Response::new(UpdateOrderResponse { status_str })),
            Err(e) => {
                tracing::error!("Error updating order: {:?}", e);
                Err(Status::internal("Failed"))
            }
        }
    }

    async fn get_all_artifacts_for_order(
        &self,
        request: Request<GetAllArtifactsForOrderPayload>,
    ) -> Result<Response<ArtifactsPurchaseDetails>, Status> {
        match utils::orders::get_all_artifacts_for_order(
            &self.db,
            request.into_inner().order_id.as_str(),
        )
        .await
        {
            Ok(artifacts) => Ok(Response::new(ArtifactsPurchaseDetails {
                buyer_id: artifacts.buyer_id,
                artifacts: artifacts.artifacts,
            })),
            Err(e) => {
                tracing::error!("Error getting order artifacts: {:?}", e);
                Err(Status::internal("Failed"))
            }
        }
    }
}
