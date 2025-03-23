use crate::{
    integration::grpc::clients::acl_service::{acl_client::AclClient, Empty},
    utils::models::AuthStatus,
};

use hyper::{
    header::{AUTHORIZATION, COOKIE},
    HeaderMap,
};
use std::io::{Error, ErrorKind};
use tonic::metadata::MetadataValue;

/// False middleware for checking authentication from ACL service for GraphQL requests.
/// I used this anti-pattern because the middleware in async-graphql just doesn't work. The headers are not properly parsed.
pub async fn check_auth_from_acl(headers: &HeaderMap) -> Result<AuthStatus, Error> {
    let mut acl_grpc_client = AclClient::connect("http://[::1]:50051")
        .await
        .map_err(|e| {
            tracing::error!("Failed to connect to ACL service: {}", e);
            Error::new(ErrorKind::Other, "Failed to connect to ACL service")
        })?;

    let mut request = tonic::Request::new(Empty {});

    let auth_header = headers.get(AUTHORIZATION);
    let cookie_header = headers.get(COOKIE);

    if auth_header.is_some() {
        let token: MetadataValue<_> = auth_header
            .unwrap()
            .to_str()
            .unwrap()
            .parse()
            .map_err(|_e| Error::new(ErrorKind::PermissionDenied, "Unauthorized"))?;

        request.metadata_mut().insert("authorization", token);
    } else {
        return Err(Error::new(ErrorKind::PermissionDenied, "Unauthorized"));
    };

    if cookie_header.is_some() {
        let cookie: MetadataValue<_> = cookie_header
            .unwrap()
            .to_str()
            .unwrap()
            .parse()
            .map_err(|_e| Error::new(ErrorKind::PermissionDenied, "Unauthorized"))?;

        request.metadata_mut().insert("cookie", cookie);
    } else {
        return Err(Error::new(ErrorKind::PermissionDenied, "Unauthorized"));
    };

    let response = acl_grpc_client.check_auth(request).await;

    match response {
        Ok(response) => {
            let current_user = response.into_inner().sub;
            Ok(AuthStatus {
                sub: current_user,
                is_auth: true,
            })
        }
        Err(_e) => return Err(Error::new(ErrorKind::PermissionDenied, "Unauthorized")),
    }
}
