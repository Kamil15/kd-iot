use embedded_graphics::egcircle;
use embedded_graphics::fonts::*;
use embedded_graphics::pixelcolor::*;
use embedded_graphics::prelude::*;
use embedded_graphics::primitive_style;
use embedded_graphics::primitives::Circle;
use embedded_graphics::style::PrimitiveStyle;
use embedded_graphics::style::Styled;
use rppal::gpio::InputPin;
use rppal::gpio::OutputPin;
use rppal::spi::Spi;
use ssd1680::prelude::*;

use super::ResultTable;

pub struct SpiDisplay {
    spi: Spi,
    ssd1680: Ssd1680<Spi, OutputPin, InputPin, OutputPin, OutputPin>,
}

impl SpiDisplay {
    pub fn new() -> Self {
        let gpio = rppal::gpio::Gpio::new().unwrap();

        let mut spi = rppal::spi::Spi::new(
            rppal::spi::Bus::Spi0,
            rppal::spi::SlaveSelect::Ss0,
            8_000_000,
            rppal::spi::Mode::Mode0,
        )
        .unwrap();
        let cs = gpio.get(26).unwrap().into_output();
        let busy = gpio.get(21).unwrap().into_input();
        let dc = gpio.get(16).unwrap().into_output();
        let rst = gpio.get(20).unwrap().into_output();

        let mut ssd1680 =
            Ssd1680::new(&mut spi, cs, busy, dc, rst, &mut rppal::hal::Delay).unwrap();

        Self { spi, ssd1680 }
    }

    pub fn update(&mut self, result_table: ResultTable) {
        self.ssd1680.clear_bw_frame(&mut self.spi).unwrap();
        let mut display_bw = Display2in13::bw();

        display_bw.set_rotation(ssd1680::graphics::DisplayRotation::Rotate270);

        let temperature = (result_table.aht20_temp + result_table.bmp280_temp + result_table.dht22_temp) / 3f32;
        let humidity = (result_table.aht20_humidity + result_table.dht22_humidity) / 2f32;

        draw_text(&mut display_bw, &format!("Temperature: {:.2} C", temperature), 0, 0);
        draw_text(&mut display_bw, &format!("Humidity: {:.2} %", humidity), 0, 17);
        draw_text(&mut display_bw, &format!("Pressure: {:.2} hPa", result_table.bmp280_pressure), 0, 34);


        let style_demo = if result_table.demo_switch {
            primitive_style!(stroke_color = BinaryColor::On, fill_color = BinaryColor::On, stroke_width = 2)
        } else {
            primitive_style!(stroke_color = BinaryColor::On, fill_color = BinaryColor::Off, stroke_width = 2)
        };

        let _ = egcircle!(
            center = (125, 90),
            radius = 20,
            style = style_demo,
        ).draw(&mut display_bw);
        

        self.ssd1680
            .update_bw_frame(&mut self.spi, display_bw.buffer())
            .unwrap();
        self.ssd1680
            .display_frame(&mut self.spi, &mut rppal::hal::Delay)
            .unwrap();
    }
}

fn draw_text(display: &mut ssd1680::graphics::Display2in13, text: &str, x: i32, y: i32) {
    let _ = Text::new(text, Point::new(x, y))
        .into_styled(embedded_graphics::text_style!(
            font = Font12x16,
            text_color = BinaryColor::On,
            background_color = BinaryColor::Off
        ))
        .draw(display);
}
