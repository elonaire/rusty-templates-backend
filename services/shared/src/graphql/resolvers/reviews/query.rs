use std::sync::Arc;

use async_graphql::{Context, Object, Result};
use axum::Extension;
use lib::{
    integration::foreign_key::add_foreign_key_if_not_exists,
    utils::{
        custom_error::ExtendedError,
        models::{ForeignKey, Product},
    },
};
use surrealdb::{engine::remote::ws::Client, Surreal};

use crate::graphql::schemas::reviews::{AverageRating, Review};

#[derive(Default)]
pub struct ReviewsQuery;

#[Object]
impl ReviewsQuery {
    async fn fetch_product_reviews(
        &self,
        ctx: &Context<'_>,
        product_id: String,
    ) -> Result<Vec<Review>> {
        let db = ctx.data::<Extension<Arc<Surreal<Client>>>>().unwrap();

        let product_fk = ForeignKey {
            table: "product_id".into(),
            column: "product_id".into(),
            foreign_key: product_id,
        };

        let product_fk_result = add_foreign_key_if_not_exists::<
            Extension<Arc<Surreal<Client>>>,
            Product,
        >(db, product_fk)
        .await;

        let mut reviews_query = db
            .query(
                "
                BEGIN TRANSACTION;
                LET $product_id = type::thing($internal_product_id);
                LET $reviews = SELECT * FROM review WHERE ->(product_id WHERE id = $product_id);
                RETURN $reviews;
                COMMIT TRANSACTION;
                ",
            )
            .bind(("internal_product_id", product_fk_result.unwrap().id))
            .await
            .map_err(|e| {
                tracing::error!("DB Query Error: {}", e);
                ExtendedError::new("Error fetching ratings", Some(400.to_string())).build()
            })?;

        let reviews: Vec<Review> = reviews_query.take(0).map_err(|e| {
            tracing::error!("Deserialization Error: {}", e);
            ExtendedError::new("Error fetching reviews", Some(400.to_string())).build()
        })?;
        Ok(reviews)
    }

    async fn fetch_product_average_rating(
        &self,
        ctx: &Context<'_>,
        product_id: String,
    ) -> Result<AverageRating> {
        let db = ctx.data::<Extension<Arc<Surreal<Client>>>>().unwrap();

        let product_fk = ForeignKey {
            table: "product_id".into(),
            column: "product_id".into(),
            foreign_key: product_id.clone(),
        };

        let product_fk_result = add_foreign_key_if_not_exists::<
            Extension<Arc<Surreal<Client>>>,
            Product,
        >(db, product_fk)
        .await;

        let mut average_rating_query = db
            .query(
                "
                BEGIN TRANSACTION;
                LET $ratings = (SELECT * FROM ONLY average_rating WHERE product_id = $external_product_id LIMIT 1);
                RETURN $ratings;
                COMMIT TRANSACTION;
                ",
            )
            .bind(("external_product_id", product_fk_result.unwrap().product_id))
            .await
            .map_err(|e| {
                tracing::error!("DB Query Error: {}", e);
                ExtendedError::new("Error fetching ratings", Some(400.to_string())).build()
            })?;

        let average_rating: Option<AverageRating> = average_rating_query.take(0).map_err(|e| {
            tracing::error!("Deserialization Error: {}", e);
            ExtendedError::new("Error fetching ratings", Some(400.to_string())).build()
        })?;

        match average_rating {
            Some(average_rating) => Ok(average_rating),
            None => Ok(AverageRating {
                product_id,
                average_rating_value: f64::default(),
                no_of_reviews: u32::default(),
            }),
        }
    }
}
