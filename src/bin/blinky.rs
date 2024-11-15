#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use esp_backtrace as _;
use esp_hal::{
    gpio::Io,
    peripherals::SPI2,
    prelude::*,
    spi::{master::Spi, SpiMode},
    timer::timg::TimerGroup,
};
use esp_println::println;
use smart_leds::{SmartLedsWrite, RGB8};
use ws2812_spi::Ws2812;

#[esp_hal_embassy::main]
async fn main(_spawner: Spawner) {
    let peripherals = esp_hal::init(esp_hal::Config::default());
    let timg0 = TimerGroup::new(peripherals.TIMG0);
    esp_hal_embassy::init(timg0.timer0);

    println!("Booting Rust Synth");
    let io = Io::new(peripherals.GPIO, peripherals.IO_MUX);

    // Make sure to bridge the solder pads labelled RGB next to the LED
    let spi: Spi<'_, SPI2, _> =
        Spi::new(peripherals.SPI2, 2.MHz(), SpiMode::Mode0).with_mosi(io.pins.gpio48);

    let mut ws = Ws2812::new(spi);

    loop {
        println!("on");
        Timer::after(Duration::from_millis(500)).await;
        ws.write([RGB8 { r: 255, g: 0, b: 0 }].into_iter()).unwrap();

        println!("off");
        Timer::after(Duration::from_millis(500)).await;
        ws.write([RGB8 { r: 0, g: 0, b: 0 }].into_iter()).unwrap();
    }
}
