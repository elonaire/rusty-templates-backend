use std::{env, time::Duration};

use rumqttc::v5::{AsyncClient, EventLoop, MqttOptions};

pub struct MqttClient;

impl MqttClient {
    pub async fn new(id: &str, host: &str, port: u16) -> (AsyncClient, EventLoop) {
        let mosquitto_user =
            env::var("MOSQUITTO_USER").expect("Missing the MOSQUITTO_USER environment variable.");
        let mosquitto_user_password = env::var("MOSQUITTO_USER_PASSWORD")
            .expect("Missing the MOSQUITTO_USER_PASSWORD environment variable.");

        let mut mqttoptions = MqttOptions::new(id, host, port);
        mqttoptions.set_keep_alive(Duration::from_secs(5));
        mqttoptions.set_credentials(mosquitto_user.as_str(), mosquitto_user_password.as_str());
        AsyncClient::new(mqttoptions, 10)
    }
}
