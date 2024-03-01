use std::{fs, time::Duration};

use prost::Message;
use rumqttc::{
    AsyncClient, Event, EventLoop, Incoming, MqttOptions, QoS, TlsConfiguration, Transport,
};
use tokio::{sync::mpsc::Receiver, task::JoinHandle};

use crate::proto::proto_broker_msgs;

use super::{ProgramArgs, ResultTable};

pub struct NetConnector {
    thread_handle: JoinHandle<()>,
    pub client: AsyncClient,
    pub receiver: Receiver<i32>,
    id_device: String,
    hostname: String,
}

impl NetConnector {
    pub async fn start_thread(args: ProgramArgs) -> NetConnector {
        println!("Start thread, args: {:?}", args);
        let id_device = args.id_device.clone();
        let hostname = args.hostname_mqqt.clone();

        let mut mqttoptions = MqttOptions::new(id_device.clone(), hostname.clone(), args.port_mqqt);

        mqttoptions.set_credentials("theserver", "myserverpass");
        mqttoptions
            .set_keep_alive(Duration::from_secs(5))
            .set_pending_throttle(Duration::from_secs(2));

        let (client, mut connection) = AsyncClient::new(mqttoptions, 10);

        client
            .subscribe(format!("iot/{}/receive", id_device), QoS::AtMostOnce)
            .await
            .unwrap();
        client
            .subscribe("iot/global", QoS::AtMostOnce)
            .await
            .unwrap();

        let (ts, receiver) = tokio::sync::mpsc::channel::<i32>(5);

        let thread_handle = tokio::spawn(async move {
            let sender = ts;
            loop {
                let notification = connection.poll().await;
                println!("Notification: {:?}", notification);
                if let Ok(Event::Incoming(Incoming::Publish(packet))) = notification {
                    println!("Incoming message!");
                    println!("{:?}", packet);
                }
            }
        });

        NetConnector {
            thread_handle,
            client,
            receiver,
            id_device,
            hostname,
        }
    }

    pub async fn send_data(&self, result_table: ResultTable) {
        let message = proto_broker_msgs::IoTMessage {
            id_device: self.id_device.clone(),
            hum: result_table.aht20_humidity,
            pressure: result_table.bmp280_pressure,
            temp: result_table.aht20_temp,
        };
        let body = message.encode_to_vec();
        let topic = format!("iot/{}/sendtoserver", self.id_device);
        self.client
            .publish(topic, QoS::AtLeastOnce, false, body)
            .await
            .unwrap();
    }

    pub fn stop(self) {
        println!("Aborting net_connector");
        self.thread_handle.abort();
        println!("Aborted net_connector");
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
