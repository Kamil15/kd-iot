use std::{borrow::BorrowMut, error::Error};

use rppal::gpio::Gpio;

use super::{net_connector::NetConnector, ResultTable};

pub struct Engine {
    dht22_fs_temp: String,
    dht22_fs_humidity: String,
    net_connector: Option<NetConnector>,
    result_table: ResultTable,
}

impl Engine {
    async fn run(&mut self) {
        self.dht22_fs_temp = "/sys/bus/iio/devices/iio:device0/in_temp_input".to_string();
        self.dht22_fs_humidity =
            "/sys/bus/iio/devices/iio:device0/in_humidityrelative_input".to_string();

        self.net_connector = Some(NetConnector::connect().await);
        self.poll_net().await;

        loop {
            self.get_dht22();
            let _ = self.get_aht20();
            let _ = self.get_bmp280();

            self.poll_net().await;
        }
    }

    /// poll when net_connector exists
    async fn poll_net(&mut self) -> Option<()> {
        self.net_connector.as_mut()?.poll().await;
        None
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
        let i2c = rppal::i2c::I2c::new()?;

        let mut aht20 = aht20::Aht20::new(i2c, rppal::hal::Delay)?;
        let (h, t) = aht20.read()?;
        self.result_table.aht20_temp = t.celsius();
        self.result_table.aht20_humidity = h.rh();

        Ok(())
    }

    pub fn get_bmp280(&mut self) -> Result<(), Box<dyn Error>> {
        let mut bmp280 = bmp280::Bmp280Builder::new()
            .address(0x20) // Optional
            .path("/dev/i2c-1") // Optional
            .build()
            .expect("Could not build device");

        self.result_table.bmp280_temp = bmp280.temperature_celsius()?;
        self.result_table.bmp280_pressure = bmp280.pressure_kpa()?;

        Ok(())
    }
}
