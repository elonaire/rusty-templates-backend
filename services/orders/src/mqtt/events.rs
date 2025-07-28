use std::sync::Arc;

use lib::utils::models::OrderStatus;
use rumqttc::v5::{mqttbytes::v5::Packet, Event};
use surrealdb::{engine::remote::ws::Client, Surreal};

use crate::utils::orders::update_order;

pub async fn handle_events(db: &Arc<Surreal<Client>>, event: &Event) -> () {
    match event {
        Event::Incoming(packet) => {
            // Handle Incoming event
            match packet {
                Packet::Publish(message) => {
                    // Handle Publish event
                    match message.topic.as_ref() {
                        b"payment/successful" => {
                            let payload_str = String::from_utf8_lossy(&message.payload);
                            // Add logic for successful payment
                            update_order(db, payload_str.as_ref(), OrderStatus::Completed)
                                .await
                                .map_err(|e| {
                                    tracing::error!(
                                        "(payment/successful)Failed to update order: {}",
                                        e
                                    );
                                })
                                .ok();
                        }
                        _ => {
                            tracing::error!("Unknown topic: {:?}", message.topic);
                            // Handle other topics
                        }
                    }
                }
                _ => {}
            }
        }
        Event::Outgoing(_) => {
            // Handle Outgoing event
        }
    }
}
