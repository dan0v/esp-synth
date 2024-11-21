#![no_std]
#![no_main]

use embassy_executor::Spawner;
use esp_backtrace as _;
use esp_println::println;

#[esp_hal_embassy::main]
async fn main(_spawner: Spawner) {
    let _ = esp_hal::init(esp_hal::Config::default());
    println!("nothing here yet!");
}

