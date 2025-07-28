use rumqttc::v5::{mqttbytes::QoS, AsyncClient};

pub async fn register_subscriptions(client: &AsyncClient) -> () {
    client
        .subscribe("payment/successful", QoS::ExactlyOnce)
        .await
        .map_err(|e| {
            tracing::error!("Failed to subscribe to payment/successful event: {}", e);
        })
        .ok();
}
