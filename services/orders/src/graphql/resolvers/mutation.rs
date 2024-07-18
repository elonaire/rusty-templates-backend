use async_graphql::MergedObject;

use super::cart::mutation::CartMutation;

#[derive(MergedObject, Default)]
pub struct Mutation(CartMutation);
