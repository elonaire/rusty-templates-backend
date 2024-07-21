use async_graphql::MergedObject;

use super::{comments::mutation::CommentMutation, ratings::mutation::RatingMutation};

#[derive(MergedObject, Default)]
pub struct Mutation(CommentMutation, RatingMutation);
