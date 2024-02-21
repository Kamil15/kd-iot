use std::{time::Duration, error::Error};

use rumqttc::*;

use crate::Args;

async fn test_dht22() -> Result<(), Box<dyn Error>> {
    use dht_embedded::{Dht22, DhtSensor, NoopInterruptControl};
    let gpio = rppal::gpio::Gpio::new().unwrap();
    let pin = gpio.get(4)?.into_io(rppal::gpio::Mode::Output);
    let mut sensor = Dht22::new(NoopInterruptControl, rppal::hal::Delay, pin);
    
    loop {
        match sensor.read() {
            Ok(reading) => println!("{}°C, {}% RH", reading.temperature(), reading.humidity()),
            Err(e) => eprintln!("Error: {}", e),
        }

        tokio::time::sleep(Duration::from_millis(2100)).await;
    }
    
    //Ok(())
}

async fn test_i2c() -> Result<(), Box<dyn Error>> {
    let i2c = rppal::i2c::I2c::new()?;
    //let mut mock_delay = rppal::hal::Delay::new();
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

/*
fn ssd1680_test() {
    use ssd1680::prelude::*;
    // use epd_waveshare::{epd1in54::*, prelude::*};
    let gpio = rppal::gpio::Gpio::new().unwrap();
    
    let mut spi = rppal::spi::Spi::new(rppal::spi::Bus::Spi0, rppal::spi::SlaveSelect::Ss0, 8_000_000, rppal::spi::Mode::Mode0).unwrap();
    let cs = gpio.get(26).unwrap().into_output();
    let busy = gpio.get(21).unwrap().into_input();
    let dc = gpio.get(16).unwrap().into_output();
    let rst = gpio.get(20).unwrap().into_output();
    

    let mut ssd1680 = Ssd1680::new(&mut spi, cs, busy, dc, rst, &mut rppal::hal::Delay).unwrap();
    ssd1680.clear_bw_frame(&mut spi).unwrap();
    let mut display_bw = Display2in13::bw();
    display_bw.set_rotation(ssd1680::graphics::DisplayRotation::Rotate270);


    let text = "a".encode_utf16();

    draw_text(&mut display_bw, "Dzien dobry.", 0, 0);
    

    ssd1680.update_bw_frame(&mut spi, display_bw.buffer()).unwrap();
    ssd1680.display_frame(&mut spi, &mut rppal::hal::Delay).unwrap();

}
fn draw_text(display: &mut ssd1680::graphics::Display2in13, text: &str, x: i32, y: i32) {
    use embedded_graphics::prelude::*;
    use embedded_graphics::fonts::*;
    use embedded_graphics::pixelcolor::*;

    

    let _ = Text::new(text, Point::new(x, y))
        .into_styled(embedded_graphics::text_style!(
            font = Font12x16,
            text_color = BinaryColor::On,
            background_color = BinaryColor::Off
        ))
        .draw(display);
}  */

async fn mqtt_load(args: Args) {
    let mut mqttoptions = MqttOptions::new(args.id_device, "localhost", 8883);

    /* let ca: Vec<u8> = fs::read("ca_certificate.pem")
        .expect("Something went wrong reading certificate!");
    mqttoptions.set_transport(Transport::Tls(TlsConfiguration::Simple {
        ca: ca,
        alpn: None,
        client_auth: None,
    })); */

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