use async_graphql::MergedObject;

use super::products::mutation::ProductMutation;

#[derive(MergedObject, Default)]
pub struct Mutation(ProductMutation);
