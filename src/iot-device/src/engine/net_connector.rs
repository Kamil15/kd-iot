use std::{fs, time::Duration};

use rumqttc::{AsyncClient, EventLoop, MqttOptions, QoS, TlsConfiguration, Transport};

pub struct NetConnector {
    client: AsyncClient,
    connection: EventLoop,
}

impl NetConnector {
    pub fn new(client: AsyncClient, connection: EventLoop) -> Self { Self { client, connection } }

    pub async fn connect() -> NetConnector {
        let mut mqttoptions = MqttOptions::new("air", "localhost", 8883);

        mqttoptions.set_credentials("iotdevice", "IttrulyisanioTdevice");
        mqttoptions
            .set_keep_alive(Duration::from_secs(5))
            .set_pending_throttle(Duration::from_secs(2));

        let (client, connection) = AsyncClient::new(mqttoptions, 10);
        
        client.subscribe("device/air", QoS::AtMostOnce).await.unwrap();
        

        NetConnector {
            client,
            connection
        }
    }

    pub async fn poll(&mut self) {
        let event = self.connection.poll().await.unwrap();
        
    }
}

fn set_cert(mut mqttoptions: MqttOptions) -> MqttOptions {
    let ca: Vec<u8> =
        fs::read("ca_certificate.pem").expect("Something went wrong reading certificate!");
    mqttoptions.set_transport(Transport::Tls(TlsConfiguration::Simple {
        ca: ca,
        alpn: None,
        client_auth: None,
    }));
    mqttoptions
}