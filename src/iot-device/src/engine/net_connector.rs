use std::{
    fs,
    sync::Arc,
    time::{Duration, SystemTime},
};

use clap::builder::Str;
use prost::Message;
use prost_types::Timestamp;
use rumqttc::{
    AsyncClient, ConnAck, ConnectReturnCode, Event, EventLoop, Incoming, MqttOptions, Packet,
    PubAck, QoS, TlsConfiguration, Transport,
};
use tokio::{
    sync::mpsc::Receiver,
    task::{self, JoinHandle},
};

use crate::proto::proto_broker_msgs::{self, ServerMessage};

use super::{ProgramArgs, ResultTable};

pub struct NetConnector {
    thread_handle: JoinHandle<()>,
    pub client: AsyncClient,
    pub receiver: Receiver<ServerMessage>,
    settings: NetConnectorSettings,
}

impl NetConnector {
    pub async fn start_thread(settings: NetConnectorSettings) -> NetConnector {
        println!("Start thread, args: {:?}", settings);

        let mut mqttoptions = MqttOptions::new(
            settings.id_device.clone(),
            settings.host.clone(),
            settings.port,
        );

        mqttoptions.set_credentials(settings.username.clone(), settings.password.clone());
        mqttoptions
            .set_keep_alive(Duration::from_secs(5))
            .set_pending_throttle(Duration::from_secs(2));

        let (client, mut connection) = AsyncClient::new(mqttoptions, 0);
        let (ts, receiver) = tokio::sync::mpsc::channel::<ServerMessage>(5);

        let move_client = client.clone();
        let move_settings = settings.clone();
        let thread_handle = tokio::spawn(async move {
            let client = move_client;
            let settings = move_settings;
            let sender = ts;
            loop {
                let notification = connection.poll().await;
                println!("Notification: {:?}", notification);
                match notification {
                    Err(_) => {
                        tokio::time::sleep(Duration::from_secs(5)).await;
                        continue;
                    }
                    //If the connection is successful but there is no existing session
                    Ok(Event::Incoming(Packet::ConnAck(ConnAck {
                        session_present: false,
                        code: ConnectReturnCode::Success,
                    }))) => {
                        //register subscribe, because there is no existing session
                        register_subscribe(client.clone(), settings.id_device.clone());
                    }
                    Ok(Event::Incoming(Incoming::Publish(packet))) => {
                        println!("Incoming message!");
                        println!("{:?}", packet);

                        if let Ok(res) =
                            proto_broker_msgs::ServerMessage::decode(packet.payload.clone())
                        {
                            println!("ServerMessage: {:?}", res);
                            sender
                                .send_timeout(res, Duration::from_secs(5))
                                .await
                                .unwrap();
                        }
                    }
                    _ => (),
                }
            }
        });

        //register_subscribe(client.clone(), settings.id_device.clone());

        NetConnector {
            thread_handle,
            client,
            receiver,
            settings,
        }
    }

    pub async fn send_data(&self, result_table: ResultTable) {
        println!("Sending data via MQTT...\n");

        let message = proto_broker_msgs::TelemetryMessage {
            id_device: self.settings.id_device.clone(),
            humidity: result_table.aht20_humidity,
            pressure: result_table.bmp280_pressure,
            temperature: result_table.aht20_temp,
            timestamp: Some(SystemTime::now().into()),
        };
        let body = message.encode_to_vec();
        let topic = format!("iotserver/{}/sendtelemetry", self.settings.id_device);

        let publish_result = self
            .client
            .try_publish(topic, QoS::AtLeastOnce, false, body);

        if let Err(error) = publish_result {
            println!("try_publish_error: {:?}", error);
        }
    }

    pub fn stop(self) {
        println!("Aborting net_connector");
        self.thread_handle.abort();
        println!("Aborted net_connector");
    }
}

//in the new task because it can block if the inner receiver is full, making it a problem if this function has been used in main loop
fn register_subscribe(client: AsyncClient, id_device: String) {
    task::spawn(async move {
        client
            .subscribe(format!("iot/{}/receive", id_device), QoS::AtMostOnce)
            .await
            .unwrap();
        client
            .subscribe("iot/global", QoS::AtMostOnce)
            .await
            .unwrap();
    });
}

#[derive(Debug, Clone)]
pub struct NetConnectorSettings {
    pub id_device: String,
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
}

impl NetConnectorSettings {
    pub fn new(
        id_device: String,
        host: String,
        port: u16,
        username: String,
        password: String,
    ) -> NetConnectorSettings {
        NetConnectorSettings {
            id_device,
            host,
            port,
            username,
            password,
        }
    }
}

#[allow(dead_code)]
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
