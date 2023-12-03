use std::{fs, time::Duration};

use clap::Parser;
use rumqttc::{Client, MqttOptions, QoS, TlsConfiguration, Transport, AsyncClient};

#[derive(Parser, Debug)]
struct Args {
    #[arg(short, long, default_value = "air")]
    id: String
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    mqtt_load(args).await;
}

async fn mqtt_load(args: Args) {
    let ca: Vec<u8> = fs::read("ca_certificate.pem")
        .expect("Something went wrong reading certificate!");
    let mut mqttoptions = MqttOptions::new(args.id, "localhost", 8883);
    mqttoptions.set_transport(Transport::Tls(TlsConfiguration::Simple {
        ca: ca,
        alpn: None,
        client_auth: None,
    }));
    mqttoptions.set_credentials("iotdevice", "IttrulyisanioTdevice");
    mqttoptions
        .set_keep_alive(Duration::from_secs(5))
        .set_pending_throttle(Duration::from_secs(2));

    let (mut client, mut connection) = AsyncClient::new(mqttoptions, 10);
    client.publish("ServerRoute", QoS::AtLeastOnce, false, "My Text").await.unwrap();
    
    // Iterate to poll the eventloop for connection progress
    loop {
        let notification = connection.poll().await.unwrap();
        println!("Notification = {:?}", notification);
    }
}
