use std::convert::Infallible;

use embedded_graphics::mono_font::iso_8859_2::FONT_10X20;
use embedded_graphics::mono_font::iso_8859_2::FONT_6X10;
use embedded_graphics::mono_font::MonoTextStyle;
use embedded_graphics::mono_font::MonoTextStyleBuilder;
use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::*;
use embedded_graphics::text::Baseline;
use embedded_graphics::text::Text;
use rppal::gpio::InputPin;
use rppal::gpio::OutputPin;
use rppal::spi::Spi;
//use ssd1680::prelude::*;
use sh1106::prelude::*;
use sh1106::NoOutputPin;

use super::ResultTable;

pub struct SpiDisplay {
    display: GraphicsMode<SpiInterface<Spi, OutputPin, OutputPin>>,
    rst: OutputPin,
}

impl SpiDisplay {
    pub fn new() -> Self {
        let gpio = rppal::gpio::Gpio::new().unwrap();

        let spi = rppal::spi::Spi::new(
            rppal::spi::Bus::Spi0,
            rppal::spi::SlaveSelect::Ss0,
            8_000_000,
            rppal::spi::Mode::Mode0, //Mode0
        )
        .unwrap();
        let cs = gpio.get(12).unwrap().into_output(); //26
        //let cs = sh1106::builder::NoOutputPin::new();
        let dc = gpio.get(16).unwrap().into_output();
        let rst = gpio.get(20).unwrap().into_output();

        let mut display: GraphicsMode<_> = sh1106::Builder::new().connect_spi(spi, dc, cs).into();

        display.init().unwrap();
        display.flush().unwrap();

        Self { display, rst }
    }

    pub fn update(&mut self, result_table: ResultTable) {
        //self.display.init().unwrap();

        // Create a new character style
        let text_style = MonoTextStyleBuilder::new()
            .font(&FONT_10X20)
            .text_color(BinaryColor::On)
            .build();
        // let style = MonoTextStyle::new(&FONT_10X20, embedded_graphics::pixelcolor::BinaryColor::On);

        // Create a text at position (20, 30) and draw it using the previously defined style
        Text::with_baseline("Hello world!", Point::zero(), text_style, Baseline::Top)
            .draw(&mut self.display)
            .unwrap();

        Text::with_baseline("Hello Rust!", Point::new(0, 16), text_style, Baseline::Top)
            .draw(&mut self.display)
            .unwrap();

        self.display.flush().unwrap();
        
        println!("Updated6");
    }
}
