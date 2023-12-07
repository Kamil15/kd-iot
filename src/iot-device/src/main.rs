use std::{fs, time::Duration, error::Error};

use clap::Parser;
use rumqttc::{Client, MqttOptions, QoS, TlsConfiguration, Transport, AsyncClient};

pub mod engine;

#[derive(Parser, Debug)]
struct Args {
    #[arg(short, long, default_value = "air")]
    id: String
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    test_perp(args).await;
    epd_waveshare_test();
}

async fn test_perp(args: Args) -> Result<(), Box<dyn Error>> {
    let mut mock_delay = rppal::hal::Delay::new();

    let mut i2c = rppal::i2c::I2c::new()?;
    //let mut aht20_uninit = aht20_driver::AHT20::new(i2c, aht20_driver::SENSOR_ADDRESS);
    //let mut aht20 = aht20_uninit.init(&mut mock_delay).unwrap();
    //let measurement = aht20.measure(&mut mock_delay).unwrap();
    //println!("temperature (aht20): {:.2}C", measurement.temperature);
    //println!("humidity (aht20): {:.2}%", measurement.humidity);

    let mut aht = aht20::Aht20::new(i2c, rppal::hal::Delay).unwrap();
    let (h, t) = aht.read().unwrap();
    println!("temperature (aht20): {:.2}C", t.celsius());
    println!("humidity (aht20): {:.2}%", h.rh());

    let mut bmp280 = bmp280::Bmp280Builder::new()
        .address(0x20) // Optional
        .path("/dev/i2c-1") // Optional
        .build().expect("Could not build device");

        println!("temperature (bmp280): {:.2}C", bmp280.temperature_celsius()?);
        println!("pressure_kpa (bmp280): {:.2}%", bmp280.pressure_kpa()?);


    Ok(())
}

fn epd_waveshare_test() {
    use ssd1680::prelude::*;
    use epd_waveshare::{epd1in54::*, prelude::*};
    let mut delay = rppal::hal::Delay::new();
    let gpio = rppal::gpio::Gpio::new().unwrap();
    
    let mut spi = rppal::spi::Spi::new(rppal::spi::Bus::Spi0, rppal::spi::SlaveSelect::Ss0, 14, rppal::spi::Mode::Mode0).unwrap();
    let cs = gpio.get(26).unwrap().into_output();
    let busy = gpio.get(21).unwrap().into_input();
    let dc = gpio.get(16).unwrap().into_output();
    let rst = gpio.get(20).unwrap().into_output();
    

    let mut ssd1680 = Ssd1680::new(&mut spi, cs, busy, dc, rst, &mut delay).unwrap();
    ssd1680.clear_bw_frame(&mut spi).unwrap();
    let mut display_bw = Display2in13::bw();
    display_bw.set_rotation(ssd1680::graphics::DisplayRotation::Rotate0);

    draw_text(&mut display_bw, "XYZ", 12, 12);

    ssd1680.update_bw_frame(&mut spi, display_bw.buffer()).unwrap();
    ssd1680.display_frame(&mut spi, &mut rppal::hal::Delay).unwrap();

}

async fn mqtt_load(args: Args) {
    let ca: Vec<u8> = fs::read("ca_certificate.pem")
        .expect("Something went wrong reading certificate!");
    let mut mqttoptions = MqttOptions::new(args.id, "localhost", 8883);
    mqttoptions.set_transport(Transport::Tls(TlsConfiguration::Simple {
        ca: ca,
        alpn: None,
        client_auth: None,
    }));
    mqttoptions.set_credentials("iotdevice", "IttrulyisanioTdevice");
    mqttoptions
        .set_keep_alive(Duration::from_secs(5))
        .set_pending_throttle(Duration::from_secs(2));

    let (mut client, mut connection) = AsyncClient::new(mqttoptions, 10);
    client.publish("ServerRoute", QoS::AtLeastOnce, false, "My Text").await.unwrap();
    
    // Iterate to poll the eventloop for connection progress
    loop {
        let notification = connection.poll().await.unwrap();
        println!("Notification = {:?}", notification);
    }
}

fn draw_text(display: &mut ssd1680::graphics::Display2in13, text: &str, x: i32, y: i32) {
    use embedded_graphics::prelude::*;

    let _ = embedded_graphics::fonts::Text::new(text, embedded_graphics::geometry::Point::new(x, y))
        .into_styled(embedded_graphics::text_style!(
            font = embedded_graphics::fonts::Font6x8,
            text_color = embedded_graphics::pixelcolor::BinaryColor::On,
            background_color = embedded_graphics::pixelcolor::BinaryColor::Off
        ))
        .draw(display);
}