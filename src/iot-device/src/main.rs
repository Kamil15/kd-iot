
use clap::{Parser, arg};

#[derive(Parser, Debug)]
pub struct Args {
    #[arg(short, long, default_value = "air")]
    id: String
}
pub mod engine;
pub mod functests;




#[tokio::main]
async fn main() {
    let args = Args::parse();


}