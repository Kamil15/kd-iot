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

    pub demo_switch: bool,
}

pub struct EnterTimerGuard {
    interval: Duration,
    last_enter: Instant,
    state: EnterTimerGuardState,
}

#[derive(PartialEq, Eq)]
enum EnterTimerGuardState {
    Default,
    ForceNextEnter,
}

impl EnterTimerGuard {
    pub fn new(interval: Duration) -> EnterTimerGuard {
        let last_enter = Instant::now();
        let state = EnterTimerGuardState::Default;
        EnterTimerGuard {
            interval,
            last_enter,
            state
        }
    }

    pub fn enter(&mut self) -> bool {
        if (self.last_enter.elapsed() > self.interval) || (self.state == EnterTimerGuardState::ForceNextEnter) {
            self.last_enter = Instant::now();
            self.state = EnterTimerGuardState::Default;
            return true;
        }
        false
    }
    pub fn force_next_enter(&mut self) {
        self.state = EnterTimerGuardState::ForceNextEnter;
    }
}



#[derive(Parser, Debug, Clone)]
pub struct ProgramArgs {
    #[arg(short, long, default_value = "air")]
    pub id_device: String,
    #[arg(long, default_value = "localhost")]
    pub host_mqqt: String,
    #[arg(short, long, default_value_t = 1883)]
    pub port_mqqt: u16,

    #[arg(long)]
    pub username_mqqt: Option<String>,
    #[arg(long)]
    pub password_mqqt: Option<String>,
}