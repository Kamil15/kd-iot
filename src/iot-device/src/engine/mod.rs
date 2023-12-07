use std::fmt::Display;

pub mod spidisplay;
pub mod net_connector;
pub mod engine;

#[derive(Debug, Clone, Copy)]
pub struct Dht22Error;
impl Display for Dht22Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Dht22Error")
    }
}
impl std::error::Error for Dht22Error {

}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct ResultTable {
    dht22_temp: f32,
    dht22_humidity: f32,
}