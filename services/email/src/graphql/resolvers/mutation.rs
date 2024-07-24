use async_graphql::{MergedObject, EmptyMutation};

#[derive(MergedObject, Default)]
pub struct Mutation(EmptyMutation);
