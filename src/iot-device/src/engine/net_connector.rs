use std::{fs, time::Duration};

use rumqttc::{AsyncClient, Event, EventLoop, Incoming, MqttOptions, QoS, TlsConfiguration, Transport};
use tokio::{sync::mpsc::Receiver, task::JoinHandle};

use super::{ProgramArgs, ResultTable};

pub struct NetConnector {
    thread_handle: JoinHandle<()>,
    pub client: AsyncClient,
    pub receiver: Receiver<i32>,
}

impl NetConnector {

    pub async fn start_thread(args: ProgramArgs) -> NetConnector {

        let mut mqttoptions = MqttOptions::new(args.id_device, args.hostname_mqqt, args.port_mqqt);

        mqttoptions.set_credentials("theserver", "myserverpass");
        mqttoptions
            .set_keep_alive(Duration::from_secs(5))
            .set_pending_throttle(Duration::from_secs(2));

        let (client, mut connection) = AsyncClient::new(mqttoptions, 10);
        
        client.subscribe("device/air", QoS::AtMostOnce).await.unwrap();
        
        let (ts, receiver ) = tokio::sync::mpsc::channel::<i32>(5);
        

        let thread_handle = tokio::spawn(async move {
            let sender = ts;
            loop {
                let notification = connection.poll().await;
                if let Ok(Event::Incoming(Incoming::Publish(packet))) = notification {
                    println!("{:?}", packet);
                }
            }
        });

        NetConnector {
            thread_handle,
            client,
            receiver,
        }
    }

    pub fn stop(self) {
        println!("Aborting net_connector");
        self.thread_handle.abort();
        println!("Aborted net_connector");
    }

    pub async fn send_data(&self, result_table: ResultTable) {
        let data = format!("{:?}", result_table);
        self.client.publish("ServerRoute", QoS::AtLeastOnce, false, data).await.unwrap();
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