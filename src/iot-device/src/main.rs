
use clap::{Parser, arg};

pub mod engine;
pub mod functests;
pub mod proto;

#[derive(Parser, Debug)]
pub struct Args {
    #[arg(short, long, default_value = "air")]
    id_device: String,
    #[arg(short, long, default_value = "localhost")]
    mqqt_hostname: String,
    #[arg(short, long, default_value = "8883")]
    mqqt_port: u16,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    println!("{:?}", 8_000_000);

}