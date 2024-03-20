use std::{
    collections::HashMap,
    time::{Duration, Instant},
};

use chrono::{DurationRound, NaiveDateTime, SubsecRound};
use clap::{Parser, Subcommand};
use comfy_table::Table;
use prost::Message;
use reqwest::Url;
use rumqttc::{AsyncClient, ConnAck, ConnectReturnCode, Event, Incoming, MqttOptions, Packet, QoS};
use tokio::{self, task};

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

        #[arg(long)]
        hostname: String,

        #[arg(short, long, default_value_t = 5)]
        duration: u64,

        #[arg(short, long)]
        username: Option<String>,
        #[arg(short, long)]
        password: Option<String>,
    },
    DisplayActivity {
        #[arg(long)]
        hostname: String,
    },
}

#[tokio::main]
async fn main() {
    let args = Cli::parse();
    match args.command.clone() {
        Commands::IotDev {
            id_device,
            hostname,
            duration,
            password,
            username,
        } => iotdev(args, id_device, hostname, duration, password, username)
            .await
            .unwrap(),
        Commands::DisplayActivity { hostname } => displayactitvity(args, hostname).await.unwrap(),
    }
}

async fn displayactitvity(args: Cli, hostname: String) -> Result<(), Box<dyn std::error::Error>> {
    let url = Url::parse(&format!("{}/api/DeviceActivityTable", hostname)).unwrap();

    let resp = reqwest::get(url)
        .await?
        .json::<HashMap<String, chrono::DateTime<chrono::Utc>>>()
        .await?;

    let mut sorted: Vec<(&String, &chrono::DateTime<chrono::Utc>)> = resp.iter().collect();
    sorted.sort_by(|a, b| b.1.partial_cmp(a.1).unwrap());

    let mut table = Table::new();
    table.set_header(vec!["Device Name", "Last seen"]);
    for (device_name, last_seen) in sorted {
        let now = chrono::offset::Utc::now().round_subsecs(0);
        let durat = now - last_seen.round_subsecs(0);

        let durat = humantime::format_duration(durat.to_std().unwrap());
        table.add_row(vec![&device_name.clone(), &format!("{durat}")]);
    }

    println!("{table}");

    Ok(())
}

async fn iotdev(
    args: Cli,
    id_device: String,
    hostname: String,
    waiting_duration: u64,
    username: Option<String>,
    password: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting thread, args: {:?}", args);

    let mut mqttoptions = MqttOptions::new(id_device.clone(), hostname.clone(), 1883);
    mqttoptions.set_credentials(
        username.unwrap_or("theserver".into()),
        password.unwrap_or("myserverpass".into()),
    );

    mqttoptions
        .set_keep_alive(Duration::from_secs(5))
        .set_pending_throttle(Duration::from_secs(2));

    let (client, mut connection) = AsyncClient::new(mqttoptions, 0);
    let (ts, mut receiver) = tokio::sync::mpsc::channel::<ServerMessage>(5);

    let (thread_client, thread_iddevice) = (client.clone(), id_device.clone());
    let _ = tokio::spawn(async move {
        let (client, id_device) = (thread_client, thread_iddevice);
        let sender = ts;
        loop {
            let notification = connection.poll().await;
            println!("Notification: {:?}", notification);
            match notification {
                Err(_) => {
                    tokio::time::sleep(Duration::from_secs(5)).await;
                    continue;
                }
                Ok(Event::Incoming(Packet::ConnAck(ConnAck {
                    session_present: false,
                    code: ConnectReturnCode::Success,
                }))) => {
                    //register subscribe, because there is no existing session
                    register_subscribe(client.clone(), id_device.clone());
                }
                Ok(Event::Incoming(Incoming::Publish(packet))) => {
                    println!("Incoming message!");
                    println!("{:?}", packet);

                    if let Ok(res) = ServerMessage::decode(packet.payload.clone()) {
                        println!("Received ServerMessage: {:?}", res);
                    }
                }
                _ => (),
            }
        }
    });

    //register_subscribe(client.clone(), id_device.clone());

    loop {
        let payload = proto_broker_msgs::ActivityMesssage {
            id_device: id_device.clone(),
            optional_state: true,
        }.encode_to_vec();
        let topic = format!("iotserver/{}/sendactivity", id_device);

        let _ = client.publish(topic, QoS::AtMostOnce, false, payload).await;
        tokio::time::sleep(Duration::from_secs(waiting_duration)).await;
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
