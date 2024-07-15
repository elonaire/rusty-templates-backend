mod graphql;
mod database;
mod rest;

use std::sync::Arc;

use async_graphql::{EmptySubscription, Schema};
use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use axum::{
    extract::Extension,
    http::{HeaderMap, HeaderValue},
    routing::post,
    Router,
    serve
};

use graphql::resolvers::query::Query;
use hyper::{
    header::{ACCEPT, ACCESS_CONTROL_ALLOW_CREDENTIALS, ACCESS_CONTROL_ALLOW_HEADERS, ACCESS_CONTROL_ALLOW_METHODS, ACCESS_CONTROL_ALLOW_ORIGIN, ACCESS_CONTROL_EXPOSE_HEADERS, AUTHORIZATION, CONTENT_TYPE, COOKIE, SET_COOKIE},
    Method,
};

// use serde::Deserialize;
use surrealdb::{engine::remote::ws::Client, Result, Surreal};
use tower_http::cors::CorsLayer;

use graphql::resolvers::mutation::Mutation;

type MySchema = Schema<Query, Mutation, EmptySubscription>;

async fn graphql_handler(
    schema: Extension<MySchema>,
    db: Extension<Arc<Surreal<Client>>>,
    headers: HeaderMap,
    req: GraphQLRequest,
) -> GraphQLResponse {
    let mut request = req.0;
    request = request.data(db.clone());
    request = request.data(headers.clone());
    schema.execute(request).await.into()
}

#[tokio::main]
async fn main() -> Result<()> {
    let db = Arc::new(database::connection::create_db_connection().await.unwrap());

    let schema = Schema::build(Query, Mutation, EmptySubscription).finish();

    println!("GraphiQL IDE: http://localhost:3001");

    // let shared_service_endpoint = env::var("SHARED_SERVICE")
    //                 .expect("Missing the SHARED_SERVICE environment variable.");

    let origins = [
        "http://localhost:8080".parse::<HeaderValue>().unwrap(),
        "http://localhost:3002".parse::<HeaderValue>().unwrap(),
        "http://localhost:3003".parse::<HeaderValue>().unwrap(),
        // shared_service_endpoint.as_str().parse::<HeaderValue>().unwrap(),
    ];

    let app = Router::new()
        .route("/", post(graphql_handler))
        // .route("/oauth/callback", get(oauth_handler))
        .layer(Extension(schema))
        .layer(Extension(db))
        .layer(
            CorsLayer::new()
                .allow_origin(origins)
                .allow_headers([AUTHORIZATION, ACCEPT, CONTENT_TYPE, SET_COOKIE, COOKIE, ACCESS_CONTROL_ALLOW_CREDENTIALS, ACCESS_CONTROL_ALLOW_CREDENTIALS, ACCESS_CONTROL_ALLOW_HEADERS, ACCESS_CONTROL_ALLOW_ORIGIN, ACCESS_CONTROL_ALLOW_METHODS, ACCESS_CONTROL_EXPOSE_HEADERS])
                .allow_credentials(true)
                .allow_methods(vec![Method::GET, Method::POST]),
        );

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3001").await.unwrap();
    serve(listener, app)
        .await
        .unwrap();

    Ok(())
}
