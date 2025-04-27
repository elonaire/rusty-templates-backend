use async_graphql::MergedObject;

use super::{comments::mutation::CommentMutation, reviews::mutation::ReviewMutation};

#[derive(MergedObject, Default)]
pub struct Mutation(CommentMutation, ReviewMutation);
