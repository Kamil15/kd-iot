use std::time::{Duration, Instant};

use clap::Parser;

pub mod spidisplay;
pub mod net_connector;
pub mod engine;


#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct ResultTable {
    pub dht22_temp: f32, //celsius
    pub dht22_humidity: f32, //hum %

    pub aht20_temp: f32, //celsius
    pub aht20_humidity: f32, //hum %

    pub bmp280_temp: f32, //celsius
    pub bmp280_pressure: f32, //kpa
}

pub struct EnterTimerGuard {
    interval: Duration,
    last_enter: Instant,
}

impl EnterTimerGuard {
    pub fn new(interval: Duration) -> EnterTimerGuard {
        let last_enter = Instant::now();
        EnterTimerGuard {
            interval,
            last_enter,
        }
    }

    pub fn enter(&mut self) -> bool {
        if self.last_enter.elapsed() > self.interval {
            self.last_enter = Instant::now();
            return true;
        }
        false
    }
}



#[derive(Parser, Debug, Clone)]
pub struct ProgramArgs {
    #[arg(short, long, default_value = "air")]
    pub id_device: String,
    #[arg(long, default_value = "localhost")]
    pub hostname_mqqt: String,
    #[arg(short, long, default_value_t = 1883)]
    pub port_mqqt: u16,
}