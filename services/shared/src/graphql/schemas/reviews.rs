use async_graphql::{ComplexObject, InputObject, SimpleObject};
use lib::utils::serialization::deserialize_float;
use serde::{Deserialize, Serialize};
use surrealdb::sql::Thing; // Import FromStr trait

#[derive(Clone, Debug, Serialize, Deserialize, SimpleObject, InputObject)]
#[graphql(input_name = "ReviewInput")]
#[graphql(complex)]
pub struct Review {
    #[graphql(skip)]
    pub id: Option<Thing>,
    pub rating: u32,
    pub comment: Option<String>,
}

#[ComplexObject]
impl Review {
    async fn id(&self) -> String {
        self.id.as_ref().map(|t| &t.id).expect("id").to_raw()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, SimpleObject)]
pub struct AverageRating {
    pub product_id: String,
    #[serde(deserialize_with = "deserialize_float")]
    pub average_rating_value: f64,
    pub no_of_reviews: u32,
}
