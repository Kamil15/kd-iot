
use clap::{Parser, arg};

pub mod engine;
pub mod functests;
pub mod proto;

#[derive(Parser, Debug)]
pub struct Args {
    #[arg(short, long, default_value = "air")]
    id_device: String,
    #[arg(long, default_value = "localhost")]
    hostname_mqqt: String,
    #[arg(short, long, default_value_t = 1883)]
    port_mqqt: u16,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    println!("{:?}", 8_000_000);
    functests::mqtt_load(args).await;

}