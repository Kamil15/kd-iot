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