use std::borrow::BorrowMut;

use rppal::gpio::Gpio;

use super::{Dht22Error, net_connector::NetConnector, ResultTable};




pub struct Engine {
    dht22_fs_temp: String,
    dht22_fs_humidity: String,
    net_connector: Option<NetConnector>,
    result_table: ResultTable,
}

impl Engine {

    async fn run(&mut self) {
        self.dht22_fs_temp = "/sys/bus/iio/devices/iio:device0/in_temp_input".to_string();
        self.dht22_fs_humidity = "/sys/bus/iio/devices/iio:device0/in_humidityrelative_input".to_string();

        self.net_connector = Some(NetConnector::connect().await);
        self.poll_net().await;

        

        loop {
            self.get_dht22();

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
}