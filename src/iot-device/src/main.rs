
use std::thread;
use std::time::Duration;

use clap::{Parser, arg};

use engine::{net_connector::NetConnectorSettings, ProgramArgs};

use crate::engine::ResultTable;

pub mod engine;
pub mod functests;
pub mod proto;

#[tokio::main]
async fn main() {
    let args = ProgramArgs::parse();
    let mut init_engine = engine::engine::Engine::new(args);
    init_engine.start_backgrund_tasks().await;
    init_engine.run().await;

    
    
    //functests::embedded_aht20().await.unwrap();
    //functests::test_i2c().await;
}