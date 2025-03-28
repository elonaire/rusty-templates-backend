use lib::{
    integration::foreign_key::add_foreign_key_if_not_exists,
    utils::{
        custom_traits::AsSurrealClient,
        models::{ArtifactsPurchaseDetails, ForeignKey, OrderStatus, User},
    },
};
use std::io::{Error, ErrorKind};

use crate::graphql::schemas::general::Order;

pub async fn update_order<T: Clone + AsSurrealClient>(
    db: &T,
    current_user: &str,
    order_id: &str,
    status: OrderStatus,
) -> Result<String, Error> {
    let user_fk = ForeignKey {
        table: "user_id".into(),
        column: "user_id".into(),
        foreign_key: current_user.into(),
    };

    let buyer_result: Option<User> = add_foreign_key_if_not_exists(db, user_fk).await;
    let buyer_result_clone = buyer_result.clone();
    let internal_user_id = buyer_result_clone
        .unwrap()
        .id
        .as_ref()
        .map(|t| &t.id)
        .expect("id")
        .to_raw();

    let mut existing_order_query = db
        .as_client()
        .query("SELECT * FROM order WHERE id=type::thing($id) AND in=type::thing($user_id) LIMIT 1")
        .bind(("user_id", format!("user_id:{}", internal_user_id)))
        .bind(("id", format!("order:{}", order_id)))
        .await
        .map_err(|e| {
            tracing::error!("DB Query Failed: {}", e);
            Error::new(ErrorKind::Other, "DB Query Failed")
        })?;

    let existing_order: Option<Order> = existing_order_query.take(0).map_err(|e| {
        tracing::error!("Deserialization Failed: {}", e);
        Error::new(ErrorKind::Other, "Deserialization Failed")
    })?;

    match existing_order {
        Some(order) => {
            let mut update_order_transaction = db
                .as_client()
                .query(
                    "
                BEGIN TRANSACTION;
                LET $order = type::thing($order_id);
                LET $new_order = UPDATE ONLY $order SET status = $new_status;
                RETURN $new_order;
                COMMIT TRANSACTION;
                ",
                )
                .bind((
                    "order_id",
                    format!(
                        "order:{}",
                        order.id.as_ref().map(|t| &t.id).expect("id").to_raw()
                    ),
                ))
                .bind(("new_status", status))
                .await
                .map_err(|e| {
                    tracing::error!("DB Query Failed: {}", e);
                    Error::new(ErrorKind::Other, "DB Query Failed")
                })?;

            let response: Option<Order> = update_order_transaction.take(0).map_err(|e| {
                tracing::error!("Deserialization Failed: {}", e);
                Error::new(ErrorKind::Other, "Deserialization Failed")
            })?;

            match status {
                OrderStatus::Confirmed => {
                    let mut _update_order_transaction = db
                        .as_client()
                    .query(
                        "
                        LET $order = type::thing($order_id);
                        LET $active_cart = (SELECT VALUE ->(cart WHERE archived=false) FROM ONLY $order LIMIT 1)[0];
                        LET $updated = (UPDATE ONLY $active_cart SET archived=true);

                        RETURN $updated;
                        "
                    )
                    .bind(("order_id", format!("order:{}", order.id.as_ref().map(|t| &t.id).expect("id").to_raw())))
                    .await
                    .map_err(|e| {
                        tracing::error!("DB Query Failed: {}", e);
                        Error::new(ErrorKind::Other, "DB Query Failed")
                    })?;
                }
                _ => {}
            }

            match response {
                Some(updated_order) => Ok(format!("{:?}", updated_order.status)),
                None => {
                    // Err(ExtendedError::new("Cart is empty!", Some(400.to_string())).build())
                    Err(Error::new(ErrorKind::Other, "Cart is empty!"))
                }
            }

            // Ok(response)
        }
        None => Err(Error::new(ErrorKind::Other, "Cart is empty!")),
    }
}

pub async fn get_all_artifacts_for_order<T: Clone + AsSurrealClient>(
    db: &T,
    order_id: &str,
) -> Result<ArtifactsPurchaseDetails, Error> {
    let mut order_artifacts_query = db
        .as_client()
        .query(
            "
            BEGIN TRANSACTION;
            LET $order_id = type::thing($id);
            LET $artifacts = SELECT VALUE artifact FROM cart_product WHERE in=(SELECT VALUE ->cart FROM ONLY $order_id LIMIT 1)[0];
            RETURN $artifacts;
            COMMIT TRANSACTION;
            "
        )
        .bind(("id", format!("order:{}", order_id)))
        .await
        .map_err(|e| {
            tracing::error!("DB Query Failed: {}", e);
            Error::new(ErrorKind::Other, "DB Query Failed")
        })?;

    let artifacts: Vec<String> = order_artifacts_query.take(0).map_err(|e| {
        tracing::error!("Deserialization Failed: {}", e);
        Error::new(ErrorKind::Other, "Deserialization Failed")
    })?;

    let mut buyer_id_query = db
        .as_client()
        .query(
            "
            BEGIN TRANSACTION;
            LET $order = type::thing($id);
            LET $buyer = SELECT VALUE (<-user_id.user_id)[0] FROM ONLY $order LIMIT 1;
            RETURN $buyer;
            COMMIT TRANSACTION;
            ",
        )
        .bind(("id", format!("order:{}", order_id)))
        .await
        .map_err(|e| {
            tracing::error!("DB Query Failed: {}", e);
            Error::new(ErrorKind::Other, "DB Query Failed")
        })?;

    let buyer_id: Option<String> = buyer_id_query.take(0).map_err(|e| {
        tracing::error!("Deserialization Failed: {}", e);
        Error::new(ErrorKind::Other, "Deserialization Failed")
    })?;

    let purchase_details = ArtifactsPurchaseDetails {
        buyer_id: buyer_id.unwrap_or("".to_string()),
        artifacts,
    };

    Ok(purchase_details)
}
