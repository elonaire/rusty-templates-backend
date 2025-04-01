use std::time::Instant;

use hyper::header::{AUTHORIZATION, COOKIE};
use tonic::body::BoxBody;
use tonic::codegen::http::{Request, Response};
use tonic::transport::Channel;
use tonic::Status;
use tonic_middleware::{Middleware, ServiceBound};

use crate::integration::grpc::clients::acl_service::{acl_client::AclClient, Empty};
use crate::utils::grpc::{create_grpc_client, AuthMetaData};

#[derive(Default, Clone)]
pub struct AuthMiddleware;

#[async_trait::async_trait]
impl<S> Middleware<S> for AuthMiddleware
where
    S: ServiceBound,
    S::Future: Send,
    S::Error: From<tonic::Status> + Send + 'static, // Add Error constraint
{
    async fn call(
        &self,
        mut req: Request<BoxBody>,
        mut service: S,
    ) -> Result<Response<BoxBody>, S::Error> {
        let start_time = Instant::now();
        tracing::debug!("Starting middleware");
        // Call the service. You can also intercept request from middleware.

        let auth_header = req.headers().get(AUTHORIZATION);
        let cookie_header = req.headers().get(COOKIE);

        let mut request = tonic::Request::new(Empty {});

        let auth_metadata: AuthMetaData<Empty> = AuthMetaData {
            auth_header,
            cookie_header,
            constructed_grpc_request: Some(&mut request),
        };

        let mut acl_grpc_client = create_grpc_client::<Empty, AclClient<Channel>>(
            "http://[::1]:50051",
            true,
            Some(auth_metadata),
        )
        .await
        .map_err(|e| {
            tracing::error!("Failed to connect to ACL service: {}", e);
            Status::unavailable("Failed to connect to ACL service")
        })?;

        let response = acl_grpc_client.check_auth(request).await?;

        let current_user = response.into_inner().sub;
        // Insert current user to the req extensions(response.sub)
        req.extensions_mut().insert(current_user);
        let result = service.call(req).await?;

        let elapsed_time = start_time.elapsed();
        tracing::info!("Request processed in {:?}", elapsed_time);

        Ok(result)
    }
}
