use std::{borrow::BorrowMut, error::Error, time::Duration};

use bmp280::Bmp280;
use rppal::{gpio::Gpio, hal::Delay, i2c::I2c};

use super::{net_connector::NetConnector, spidisplay::{self, SpiDisplay}, EnterTimerGuard, ProgramArgs, ResultTable};

pub struct Engine {
    args: ProgramArgs,
    dht22_fs_temp: String,
    dht22_fs_humidity: String,
    net_connector: Option<NetConnector>,
    display: SpiDisplay,
    aht20: embedded_aht20::Aht20<I2c, Delay>,
    bmp280: Bmp280,
    result_table: ResultTable,
}

impl Engine {
    pub fn new(args: ProgramArgs) -> Engine {
        let dht22_fs_temp = "/sys/bus/iio/devices/iio:device0/in_temp_input".to_string();
        let dht22_fs_humidity =
            "/sys/bus/iio/devices/iio:device0/in_humidityrelative_input".to_string();
        let net_connector = None;
        let result_table = ResultTable::default();

        let i2c = rppal::i2c::I2c::new().unwrap();
        let aht20 = embedded_aht20::Aht20::new(i2c, embedded_aht20::DEFAULT_I2C_ADDRESS, rppal::hal::Delay).unwrap();
        let bmp280 = bmp280::Bmp280Builder::new()
            .address(0x77) // Optional
            .path("/dev/i2c-1") // Optional
            .build()
            .expect("Could not build device");

        let display = SpiDisplay::new();

        Engine {
            args,
            dht22_fs_temp,
            dht22_fs_humidity,
            net_connector,
            display,
            aht20,
            bmp280,
            result_table,
        }
    }
    pub async fn start_backgrund_tasks(&mut self) {
        self.net_connector = Some(NetConnector::start_thread(self.args.clone()).await);
    }

    pub async fn run(&mut self) {
        let mut dht22_timer = EnterTimerGuard::new(Duration::from_secs(5));
        let mut aht20_timer = EnterTimerGuard::new(Duration::from_secs(5));
        let mut bmp280_timer = EnterTimerGuard::new(Duration::from_secs(5));
        let mut print_timer = EnterTimerGuard::new(Duration::from_secs(8));
        let mut spidisplay_timer = EnterTimerGuard::new(Duration::from_secs(16));

        let mut send_timer = EnterTimerGuard::new(Duration::from_secs(16));
        

        loop {
            tokio::time::sleep(Duration::from_secs(2)).await; //temp

            if dht22_timer.enter() {
                self.get_dht22();
            }

            if aht20_timer.enter() {
                let _ = self.get_aht20();
            }

            if bmp280_timer.enter() {
                let _ = self.get_bmp280();
            }

            if print_timer.enter() {
                println!("{:?}", self.result_table);
            }

            if spidisplay_timer.enter() {
                self.display.update(self.result_table);
            }

            if send_timer.enter() {
                self.net_connector.as_ref().unwrap().send_data(self.result_table.clone()).await;
            }
        }
    }

    fn get_dht22(&mut self) {
        let _ = std::fs::read_to_string(self.dht22_fs_temp.as_str()).and_then(|it| {
            let _ = it.trim_end().parse::<f32>().and_then(|parsed| {
                self.result_table.dht22_temp = parsed / 1000.0;
                Ok(())
            });
            Ok(())
        });

        let _ = std::fs::read_to_string(self.dht22_fs_humidity.as_str()).and_then(|it| {
            let _ = it.trim_end().parse::<f32>().and_then(|parsed| {
                self.result_table.dht22_humidity = parsed / 1000.0;
                Ok(())
            });
            Ok(())
        });
    }

    fn get_aht20(&mut self) -> Result<(), embedded_aht20::Error<rppal::i2c::Error>> {
        let result = self.aht20.measure()?;
        self.result_table.aht20_temp = result.temperature.celcius();
        self.result_table.aht20_humidity = result.relative_humidity;

        Ok(())
    }

    fn get_bmp280(&mut self) -> Result<(), Box<dyn Error>> {
        self.result_table.bmp280_temp = self.bmp280.temperature_celsius()?;
        self.result_table.bmp280_pressure = self.bmp280.pressure_kpa()?;

        Ok(())
    }
}
