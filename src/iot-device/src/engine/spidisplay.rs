use embedded_graphics::fonts::*;
use embedded_graphics::pixelcolor::*;
use embedded_graphics::prelude::*;
use rppal::gpio::InputPin;
use rppal::gpio::OutputPin;
use rppal::spi::Spi;
use ssd1680::prelude::*;

struct SpiDisplay {
    spi: Spi,
    ssd1680: Ssd1680<Spi, OutputPin, InputPin, OutputPin, OutputPin>,
}

impl SpiDisplay {
    fn new() -> Self {
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

    fn update(&mut self) {
        self.ssd1680.clear_bw_frame(&mut self.spi).unwrap();
        let mut display_bw = Display2in13::bw();

        display_bw.set_rotation(ssd1680::graphics::DisplayRotation::Rotate270);

        let text = "a".encode_utf16();

        draw_text(&mut display_bw, "Dzien dobry.", 0, 0);

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
