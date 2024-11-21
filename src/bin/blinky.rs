#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use esp_backtrace as _;
use esp_hal::{
    gpio::Io,
    prelude::*,
    spi::{master::Spi, SpiMode},
    timer::timg::TimerGroup,
};
use esp_println::println;
use smart_leds::{SmartLedsWrite, RGB8};
use ws2812_spi::Ws2812;

#[esp_hal_embassy::main]
async fn main(_spawner: Spawner) {
    // Initialize the system (set frequency, setup interrupts, etc.)
    let peripherals = esp_hal::init(esp_hal::Config::default());
    // Initialize the embassy runtime (embassy is the async framework we're using)
    let timg0 = TimerGroup::new(peripherals.TIMG0);
    esp_hal_embassy::init(timg0.timer0);
    // Initialize the IO driver
    let io = Io::new(peripherals.GPIO, peripherals.IO_MUX);

    // We're using the serial port interface (SPI) to control the on-board addressable LED. The SPI
    // is normally used to send data between a computer and its peripherals. Due to the SPI's ability
    // to send arbitrary bit streams, it can be used to implement other protocols by so-called
    // bit-banging, such as the addressable LED protocol.
    // Make sure the solder pads labelled RGB next to the LED are bridged
    let spi = Spi::new(peripherals.SPI2, 2.MHz(), SpiMode::Mode0).with_mosi(io.pins.gpio48);
    let mut ws = Ws2812::new(spi);

    loop {
        Timer::after(Duration::from_millis(500)).await;
        println!("on");
        ws.write([RGB8 { r: 255, g: 0, b: 0 }].into_iter()).unwrap();

        Timer::after(Duration::from_millis(500)).await;
        println!("off");
        ws.write([RGB8 { r: 0, g: 0, b: 0 }].into_iter()).unwrap();
    }
}
