mod database;
mod graphql;
mod grpc;
mod rest;
mod utils;

use std::{env, net::SocketAddr, sync::Arc};

use async_graphql::{EmptySubscription, Schema};
use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use axum::{
    extract::Extension,
    http::{HeaderMap, HeaderValue},
    routing::post,
    serve, Router,
};

use graphql::resolvers::query::Query;
use grpc::server::{
    orders_service::orders_service_server::OrdersServiceServer, OrdersServiceImplementation,
};
use hyper::{
    header::{
        ACCEPT, ACCESS_CONTROL_ALLOW_CREDENTIALS, ACCESS_CONTROL_ALLOW_HEADERS,
        ACCESS_CONTROL_ALLOW_METHODS, ACCESS_CONTROL_ALLOW_ORIGIN, ACCESS_CONTROL_EXPOSE_HEADERS,
        AUTHORIZATION, CONTENT_TYPE, COOKIE, SET_COOKIE,
    },
    Method,
};

use lib::middleware::auth::grpc::AuthMiddleware;
// use serde::Deserialize;
use surrealdb::{engine::remote::ws::Client, Result, Surreal};
use tonic::transport::Server;
use tonic_middleware::MiddlewareLayer;
use tower_http::cors::CorsLayer;

use graphql::resolvers::mutation::Mutation;
use tracing_subscriber::fmt::writer::MakeWriterExt;

type MySchema = Schema<Query, Mutation, EmptySubscription>;

/// Main GraphQL handler, all requests pass through here.
async fn graphql_handler(
    schema: Extension<MySchema>,
    db: Extension<Arc<Surreal<Client>>>,
    headers: HeaderMap,
    req: GraphQLRequest,
) -> GraphQLResponse {
    let mut request = req.0;
    request = request.data(db.clone());
    request = request.data(headers.clone());
    let operation_name = request.operation_name.clone();

    // Log request info
    tracing::info!("Executing GraphQL request: {:?}", &operation_name);
    let start = std::time::Instant::now();

    // Execute the GraphQL request
    let response = schema.execute(request).await;

    let duration = start.elapsed();
    tracing::info!("{:?} request processed in {:?}", operation_name, duration);

    // Debug the response
    if response.errors.len() > 0 {
        tracing::debug!("GraphQL Error: {:?}", response.errors);
    } else {
        tracing::info!("GraphQL request completed without errors");
    }

    // Convert GraphQL response into the Axum response type
    response.into()
}

#[tokio::main]
async fn main() -> Result<()> {
    let db = Arc::new(database::connection::create_db_connection().await.unwrap());

    // Bring in some needed env vars
    let deployment_env = env::var("ENVIRONMENT").unwrap_or_else(|_| "prod".to_string()); // default to production because it's the most secure
    let allowed_services_cors = env::var("ALLOWED_SERVICES_CORS")
        .expect("Missing the ALLOWED_SERVICES environment variable.");
    let orders_http_port =
        env::var("ORDERS_HTTP_PORT").expect("Missing the ORDERS_HTTP_PORT environment variable.");
    let orders_grpc_port =
        env::var("ORDERS_GRPC_PORT").expect("Missing the ORDERS_GRPC_PORT environment variable.");

    let mut schema_builder =
        Schema::build(Query::default(), Mutation::default(), EmptySubscription);

    // Disable introspection & limit query depth in production
    schema_builder = match deployment_env.as_str() {
        "prod" => schema_builder.disable_introspection().limit_depth(5),
        _ => schema_builder,
    };

    let schema = schema_builder.finish();

    let origins: Vec<HeaderValue> = allowed_services_cors
        .as_str()
        .split(",")
        .into_iter()
        .map(|endpoint| endpoint.parse::<HeaderValue>().unwrap())
        .collect();

    // Persist the server logs to a file on a daily basis using "tracing_subscriber"
    let file_appender = tracing_appender::rolling::daily("./logs", "orders.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    let stdout = std::io::stdout.with_max_level(tracing::Level::DEBUG); // Log to console at DEBUG level

    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .with_writer(stdout.and(non_blocking))
        .init();

    let app = Router::new()
        .route("/", post(graphql_handler))
        // .route("/oauth/callback", get(oauth_handler))
        .layer(Extension(schema))
        .layer(Extension(db.clone()))
        .layer(
            CorsLayer::new()
                .allow_origin(origins)
                .allow_headers([
                    AUTHORIZATION,
                    ACCEPT,
                    CONTENT_TYPE,
                    SET_COOKIE,
                    COOKIE,
                    ACCESS_CONTROL_ALLOW_CREDENTIALS,
                    ACCESS_CONTROL_ALLOW_CREDENTIALS,
                    ACCESS_CONTROL_ALLOW_HEADERS,
                    ACCESS_CONTROL_ALLOW_ORIGIN,
                    ACCESS_CONTROL_ALLOW_METHODS,
                    ACCESS_CONTROL_EXPOSE_HEADERS,
                ])
                .allow_credentials(true)
                .allow_methods(vec![Method::GET, Method::POST]),
        );

    // Set up the gRPC server
    let orders_grpc = OrdersServiceImplementation::new(db.clone());
    let grpc_address: SocketAddr = format!("[::1]:{}", orders_grpc_port)
        .as_str()
        .parse()
        .unwrap();
    let tonic_auth_middleware = AuthMiddleware::default();

    tokio::spawn(async move {
        // let the thread panic if gRPC server fails to start
        Server::builder()
            .layer(MiddlewareLayer::new(tonic_auth_middleware))
            .add_service(OrdersServiceServer::new(orders_grpc))
            .serve(grpc_address)
            .await
            .unwrap();
    });

    // let listener = tokio::net::TcpListener::bind("0.0.0.0:3010").await.unwrap();
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", orders_http_port))
        .await
        .unwrap();
    serve(listener, app).await.unwrap();

    Ok(())
}
