// use std::sync::Arc;

// use crate::graphql::schemas::general::Product;
// use async_graphql::{Context, Error, Object, Result};
// use axum::Extension;
// use surrealdb::{engine::remote::ws::Client, Surreal};
// use lib::{middleware::{auth::check_auth_from_acl, user::add_foreign_key_if_not_exists}, utils::{auth::{ForeignKey, User}, custom_error::ExtendedError}};

// #[derive(Default)]
// pub struct OrderMutation;

// #[Object]
// impl OrderMutation {
//     pub async fn create_order(&self, ctx: &Context<'_>, order: Order) -> Result<Vec<Order>> {}
// }
