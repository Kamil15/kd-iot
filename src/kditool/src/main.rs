use std::{collections::HashMap, time::Duration};

use clap::{Parser, Subcommand};
use prost::Message;
use reqwest::Url;
use rumqttc::{AsyncClient, Event, Incoming, MqttOptions, QoS};
use tokio;

use crate::proto::proto_broker_msgs::{self, ServerMessage};

mod proto;

#[derive(Parser, Debug, Clone)]
pub struct Cli {
    /// Turn debugging information on
    #[arg(short, long, action = clap::ArgAction::Count)]
    debug: u8,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug, Clone)]
enum Commands {
    /// does testing things
    IotDev {
        /// lists test values
        #[arg(short, long)]
        id_device: String,

        #[arg(short, long)]
        hostname: String,

        #[arg(long)]
        username: Option<String>,
        #[arg(long)]
        password: Option<String>,
    },
    DisplayActivity {
        #[arg(short, long)]
        hostname: String,
    },
}

#[tokio::main]
async fn main() {
    let args = Cli::parse();
    match args.command.clone() {
        Commands::IotDev {
            id_device,
            hostname, .. } => iotdev(args, id_device, hostname).await.unwrap(),
        Commands::DisplayActivity { hostname } => displayactitvity(args, hostname).await.unwrap(),
    }
}

async fn displayactitvity(args: Cli, hostname: String) -> Result<(), Box<dyn std::error::Error>> {
    let url = Url::parse(&format!("{}/DeviceActivityTable", hostname)).unwrap();

    let resp = reqwest::get(url)
        .await?
        .json::<HashMap<String, String>>()
        .await?;

    println!("{resp:#?}");

    Ok(())
}

async fn iotdev(
    args: Cli,
    id_device: String,
    hostname: String,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting thread, args: {:?}", args);

    let mut mqttoptions = MqttOptions::new(id_device.clone(), hostname.clone(), 1883);

    mqttoptions.set_credentials("theserver", "myserverpass");
    mqttoptions
        .set_keep_alive(Duration::from_secs(5))
        .set_pending_throttle(Duration::from_secs(2));

    let (client, mut connection) = AsyncClient::new(mqttoptions, 0);
    let (ts, mut receiver) = tokio::sync::mpsc::channel::<ServerMessage>(5);

    let _ = tokio::spawn(async move {
        let sender = ts;
        loop {
            let notification = connection.poll().await;
            println!("Notification: {:?}", notification);
            if let Err(_) = notification {
                tokio::time::sleep(Duration::from_secs(5)).await;
                continue;
            }

            if let Ok(Event::Incoming(Incoming::Publish(packet))) = notification {
                println!("Incoming message!");
                println!("{:?}", packet);

                if let Ok(res) = ServerMessage::decode(packet.payload.clone()) {
                    println!("Received ServerMessage: {:?}", res);
                }
            }
        }
    });

    client
        .subscribe(format!("iot/{}/receive", id_device), QoS::AtMostOnce)
        .await
        .unwrap();
    client
        .subscribe("iot/global", QoS::AtMostOnce)
        .await
        .unwrap();

    loop {
        let message = proto_broker_msgs::ActivityMesssage {
            id_device: id_device.clone(),
            optional_state: true,
        };
        let payload = message.encode_to_vec();
        let topic = format!("iotserver/{}/sendactivity", id_device);

        let _ = client.publish(topic, QoS::AtMostOnce, false, payload).await;
        tokio::time::sleep(Duration::from_secs(5)).await;
    }
}
