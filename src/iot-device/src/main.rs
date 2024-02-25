
use std::thread;
use std::time::Duration;

use clap::{Parser, arg};

use engine::ProgramArgs;
use engine::spidisplay::SpiDisplay;

use crate::engine::ResultTable;

pub mod engine;
pub mod functests;
pub mod proto;

#[tokio::main]
async fn main() {
    let args = ProgramArgs::parse();
    //println!("{:?}", 2_000_000);
    //let mut disp = SpiDisplay::new();
    //disp.update(ResultTable::default());
    functests::test_ssd1306();
    loop {}
}