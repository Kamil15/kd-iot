use embedded_graphics::prelude::*;
use rppal::gpio::InputPin;
use rppal::gpio::OutputPin;
use rppal::spi::Spi;
use sh1106::mode::displaymode::DisplayMode;
use sh1106::mode::RawMode;
//use ssd1680::prelude::*;
use sh1106::prelude::*;

struct SpiDisplay{
    display: GraphicsMode<SpiInterface<Spi, OutputPin, OutputPin>>,
    rst: OutputPin,
}

impl SpiDisplay {
    fn new() -> Self {
        let gpio = rppal::gpio::Gpio::new().unwrap();

        let spi = rppal::spi::Spi::new(
            rppal::spi::Bus::Spi0,
            rppal::spi::SlaveSelect::Ss0,
            8_000_000,
            rppal::spi::Mode::Mode0,
        )
        .unwrap();
        let cs = gpio.get(26).unwrap().into_output();
        let dc = gpio.get(16).unwrap().into_output();
        let rst = gpio.get(20).unwrap().into_output();

        let mut display: GraphicsMode<_> = sh1106::Builder::new().connect_spi(spi, dc, cs).into();
        
        display.init().unwrap();
        display.flush().unwrap();

        Self { display, rst }
    }

    fn update(&mut self) {

    }
}