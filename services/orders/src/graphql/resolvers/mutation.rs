use async_graphql::MergedObject;

use super::{cart::mutation::CartMutation, orders::mutation::OrderMutation};

#[derive(MergedObject, Default)]
pub struct Mutation(CartMutation, OrderMutation);
