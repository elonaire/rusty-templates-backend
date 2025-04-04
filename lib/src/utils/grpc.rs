use async_trait::async_trait;
use hyper::header::HeaderValue;
use std::io::{Error as StdError, ErrorKind};
use tonic::{
    metadata::MetadataValue,
    transport::{Channel, Endpoint, Error},
    Request,
};

use crate::integration::grpc::clients::{
    acl_service::acl_client::AclClient, email_service::email_service_client::EmailServiceClient,
    files_service::files_service_client::FilesServiceClient,
    orders_service::orders_service_client::OrdersServiceClient,
    payments_service::payments_service_client::PaymentsServiceClient,
    products_service::products_service_client::ProductsServiceClient,
};

// Define the trait for gRPC clients
#[async_trait]
pub trait GrpcClient: Sized {
    async fn connect<'a>(endpoint: &'a str) -> Result<Self, Error>;
}

pub struct AuthMetaData<'a, T> {
    pub auth_header: Option<&'a HeaderValue>,
    pub cookie_header: Option<&'a HeaderValue>,
    pub constructed_grpc_request: Option<&'a mut Request<T>>,
}

// Implement the trait for AclClient<Channel>
#[async_trait]
impl GrpcClient for AclClient<Channel> {
    async fn connect<'a>(endpoint: &'a str) -> Result<Self, Error> {
        let channel = Endpoint::from_shared(endpoint.to_string())?
            .connect()
            .await?;
        Ok(AclClient::new(channel))
    }
}

// Implement the trait for EmailServiceClient<Channel>
#[async_trait]
impl GrpcClient for EmailServiceClient<Channel> {
    async fn connect<'a>(endpoint: &'a str) -> Result<Self, Error> {
        let channel = Endpoint::from_shared(endpoint.to_string())?
            .connect()
            .await?;
        Ok(EmailServiceClient::new(channel))
    }
}

// Implement the trait for FilesServiceClient<Channel>
#[async_trait]
impl GrpcClient for FilesServiceClient<Channel> {
    async fn connect<'a>(endpoint: &'a str) -> Result<Self, Error> {
        let channel = Endpoint::from_shared(endpoint.to_string())?
            .connect()
            .await?;
        Ok(FilesServiceClient::new(channel))
    }
}

// Implement the trait for PaymentsServiceClient<Channel>
#[async_trait]
impl GrpcClient for PaymentsServiceClient<Channel> {
    async fn connect<'a>(endpoint: &'a str) -> Result<Self, Error> {
        let channel = Endpoint::from_shared(endpoint.to_string())?
            .connect()
            .await?;
        Ok(PaymentsServiceClient::new(channel))
    }
}

// Implement the trait for OrdersServiceClient<Channel>
#[async_trait]
impl GrpcClient for OrdersServiceClient<Channel> {
    async fn connect<'a>(endpoint: &'a str) -> Result<Self, Error> {
        let channel = Endpoint::from_shared(endpoint.to_string())?
            .connect()
            .await?;
        Ok(OrdersServiceClient::new(channel))
    }
}

// Implement the trait for ProductsServiceClient<Channel>
#[async_trait]
impl GrpcClient for ProductsServiceClient<Channel> {
    async fn connect<'a>(endpoint: &'a str) -> Result<Self, Error> {
        let channel = Endpoint::from_shared(endpoint.to_string())?
            .connect()
            .await?;
        Ok(ProductsServiceClient::new(channel))
    }
}

// Generic function to create gRPC clients
pub async fn create_grpc_client<'a, R, T: GrpcClient>(
    endpoint: &str,
    is_authenticated: bool,
    auth_metadata: Option<AuthMetaData<'_, R>>,
) -> Result<T, StdError> {
    if is_authenticated {
        add_auth_headers_to_request::<R>(auth_metadata.unwrap()).await?;
    }
    T::connect(endpoint)
        .await
        .map_err(|_e| StdError::new(ErrorKind::InvalidData, "Invalid header"))
}

async fn add_auth_headers_to_request<R>(
    mut auth_metadata: AuthMetaData<'_, R>,
) -> Result<(), StdError> {
    let token: MetadataValue<_> = auth_metadata
        .auth_header
        .unwrap()
        .to_str()
        .unwrap()
        .parse()
        .map_err(|e| {
            tracing::error!("Failed to parse auth header: {}", e);
            StdError::new(ErrorKind::InvalidData, "Invalid header")
        })?;

    auth_metadata
        .constructed_grpc_request
        .as_mut()
        .unwrap()
        .metadata_mut()
        .insert("authorization", token);

    let cookie: MetadataValue<_> = auth_metadata
        .cookie_header
        .unwrap()
        .to_str()
        .unwrap()
        .parse()
        .map_err(|e| {
            tracing::error!("Failed to parse cookie header: {}", e);
            StdError::new(ErrorKind::InvalidData, "Invalid header")
        })?;

    auth_metadata
        .constructed_grpc_request
        .as_mut()
        .unwrap()
        .metadata_mut()
        .insert("cookie", cookie);

    Ok(())
}
