use std::{time::Duration, error::Error};

use rumqttc::*;

use crate::ProgramArgs;

/* async fn test_dht22() -> Result<(), Box<dyn Error>> {
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
} */

pub async fn test_i2c() -> Result<(), Box<dyn Error>> {
    let i2c = rppal::i2c::I2c::new()?;

    /*let mut aht = aht20::Aht20::new(i2c, rppal::hal::Delay).unwrap();
    aht.read().unwrap();
    let (h, t) = aht.read().unwrap();
    println!("temperature (aht20): {:.2}C", t.celsius());
    println!("humidity (aht20): {:.2}%", h.rh());*/

    let mut aht = embedded_aht20::Aht20::new(i2c, embedded_aht20::DEFAULT_I2C_ADDRESS, rppal::hal::Delay).unwrap();
    let measure = aht.measure().unwrap();
    
    println!("Temperature: {:.2} °C, Relative humidity: {:.2} %", measure.temperature.celcius(), measure.relative_humidity);

    let mut bmp280 = bmp280::Bmp280Builder::new()
        .address(0x77) // Optional
        .path("/dev/i2c-1") // Optional
        .build().expect("Could not build device");

        println!("temperature (bmp280): {:.2}C", bmp280.temperature_celsius()?);
        println!("pressure_kpa (bmp280): {:.2}%", bmp280.pressure_kpa()?);


    Ok(())
}

pub async fn embedded_aht20() -> Result<(), Box<dyn Error>> {
    let i2c = rppal::i2c::I2c::new()?;

    let mut aht = embedded_aht20::Aht20::new(i2c, embedded_aht20::DEFAULT_I2C_ADDRESS, rppal::hal::Delay).unwrap();
    let measure = aht.measure().unwrap();
    
    println!("Temperature: {:.2} °C, Relative humidity: {:.2} %", measure.temperature.celcius(), measure.relative_humidity);

    Ok(())
}


pub fn ssd1680_test() {
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


    //let text = "a".encode_utf16();

    //draw_text(&mut display_bw, "Test Pierwotny.", 0, 0);
    

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
}

pub async fn mqtt_load(args: ProgramArgs) {
    let mut mqttoptions = MqttOptions::new(args.id_device, "localhost", 1883);

    /* let ca: Vec<u8> = fs::read("ca_certificate.pem")
        .expect("Something went wrong reading certificate!");
    mqttoptions.set_transport(Transport::Tls(TlsConfiguration::Simple {
        ca: ca,
        alpn: None,
        client_auth: None,
    })); */

    mqttoptions.set_credentials("theserver", "myserverpass");
    mqttoptions
        .set_keep_alive(Duration::from_secs(5))
        .set_pending_throttle(Duration::from_secs(2));

    let (mut client, mut connection) = AsyncClient::new(mqttoptions, 10);
    client.publish("ServerRoute", QoS::AtLeastOnce, false, "My Text").await.unwrap();
    
    // Iterate to poll the eventloop for connection progress
    loop {
        if let Ok(notification) = connection.poll().await {
            println!("Notification = {:?}", notification);
        }
    }
}

/*
pub fn test_ssd1306() {
    use ssd1306::prelude::*;
    use ssd1306::Ssd1306;
    use embedded_graphics::mono_font::MonoTextStyleBuilder;
    use embedded_graphics::prelude::*;
    use embedded_graphics::mono_font::ascii::FONT_6X10;
    use embedded_graphics::pixelcolor::BinaryColor;
    use embedded_graphics::text::Text;
    use embedded_graphics::text::Baseline;

    let gpio = rppal::gpio::Gpio::new().unwrap();

    let spi = rppal::spi::Spi::new(
        rppal::spi::Bus::Spi0,
        rppal::spi::SlaveSelect::Ss0,
        1_000_000,
        rppal::spi::Mode::Mode0, //Mode0
    )
    .unwrap();

    //let cs = gpio.get(12).unwrap().into_output(); //26
    let dc = gpio.get(16).unwrap().into_output();

    let interface = SPIInterfaceNoCS::new(spi, dc);
    let mut display = Ssd1306::new(
        interface,
        DisplaySize128x64,
        DisplayRotation::Rotate0,
    ).into_buffered_graphics_mode();
    display.init().unwrap();
    
    let text_style = MonoTextStyleBuilder::new()
        .font(&FONT_6X10)
        .text_color(BinaryColor::On)
        .build();
    
    Text::with_baseline("Hello world!", Point::zero(), text_style, Baseline::Top)
        .draw(&mut display)
        .unwrap();
    
    Text::with_baseline("Hello Rust!", Point::new(0, 16), text_style, Baseline::Top)
        .draw(&mut display)
        .unwrap();
    
    display.set_pixel(1, 1, true);
    display.flush().unwrap();
    println!("Done! 8");
}*/