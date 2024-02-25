use std::{borrow::BorrowMut, error::Error, time::Duration};

use bmp280::Bmp280;
use rppal::{gpio::Gpio, hal::Delay, i2c::I2c};

use super::{net_connector::NetConnector, ProgramArgs, ResultTable};

pub struct Engine {
    args: ProgramArgs,
    dht22_fs_temp: String,
    dht22_fs_humidity: String,
    net_connector: Option<NetConnector>,
    aht20: aht20::Aht20<I2c, Delay>,
    bmp280: Bmp280,
    result_table: ResultTable,
}

impl Engine {
    fn new(args: ProgramArgs) -> Engine {
        let dht22_fs_temp = "/sys/bus/iio/devices/iio:device0/in_temp_input".to_string();
        let dht22_fs_humidity =
            "/sys/bus/iio/devices/iio:device0/in_humidityrelative_input".to_string();
        let net_connector = None;
        let result_table = ResultTable::default();

        let i2c = rppal::i2c::I2c::new().unwrap();
        let aht20 = aht20::Aht20::new(i2c, rppal::hal::Delay).unwrap();
        let bmp280 = bmp280::Bmp280Builder::new()
            .address(0x77) // Optional
            .path("/dev/i2c-1") // Optional
            .build()
            .expect("Could not build device");

        Engine {
            args,
            dht22_fs_temp,
            dht22_fs_humidity,
            net_connector,
            aht20,
            bmp280,
            result_table,
        }
    }
    async fn start_backgrund_tasks(&mut self) {
        self.net_connector = Some(NetConnector::start_thread(self.args.clone()).await);
    }

    async fn run(&mut self) {
        

        loop {
            tokio::time::sleep(Duration::from_secs(5)).await; //temp
            self.get_dht22();
            let _ = self.get_aht20();
            let _ = self.get_bmp280();
            self.net_connector.as_ref().unwrap().send_data(self.result_table.clone()).await;
        }
    }

    pub fn get_dht22(&mut self) {
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

    pub fn get_aht20(&mut self) -> Result<(), aht20::Error<rppal::i2c::Error>> {
        let (h, t) = self.aht20.read()?;
        self.result_table.aht20_temp = t.celsius();
        self.result_table.aht20_humidity = h.rh();

        Ok(())
    }

    pub fn get_bmp280(&mut self) -> Result<(), Box<dyn Error>> {
        self.result_table.bmp280_temp = self.bmp280.temperature_celsius()?;
        self.result_table.bmp280_pressure = self.bmp280.pressure_kpa()?;

        Ok(())
    }
}
