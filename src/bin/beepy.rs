#![no_std]
#![no_main]

extern crate alloc;

use embassy_executor::Spawner;
use esp_backtrace as _;
use esp_hal::{
    dma::{Dma, DmaPriority},
    gpio::{ GpioPin, Io},i2s::{asynch::I2sWriteDmaAsync, I2sTx},
    timer::timg::TimerGroup,
};
use esp_println::println;
use synth::{
    i2s,
    oscillators::{scales::REFERENCE_FREQ, traits::Generator, SineOscillator},
};

#[esp_hal_embassy::main]
async fn main(_spawner: Spawner) {
    let peripherals = esp_hal::init(esp_hal::Config::default());
    let timg0 = TimerGroup::new(peripherals.TIMG0);
    esp_hal_embassy::init(timg0.timer0);
    esp_alloc::heap_allocator!(8192);

    println!("Booting Rust Synth");
    let io = Io::new(peripherals.GPIO, peripherals.IO_MUX);

    // I2S =============================
    // Set up DMA (direct memory access) buffers.
    let dma = Dma::new(peripherals.DMA);
    let dma_channel = dma
        .channel0
        .configure_for_async(false, DmaPriority::Priority9);
    let i2s = i2s::new_i2s(peripherals.I2S0, dma_channel);

    // Create the i2s transfer channel and define pins
    // The channel is used to control data flow to the DMA transaction
    let i2s_tx: I2sTx<_, _> = i2s
        .i2s_tx
        .with_bclk(todo!("Please choose a GPIO pin to use here, such as io.pins.gpio0)!") as GpioPin<0>)  // Connect to BCK on PCM5102A DAC
        .with_dout(todo!("Please choose a GPIO pin to use here, such as io.pins.gpio0!") as GpioPin<0>)  // Connect to DIN on PCM5102A DAC
        .with_ws(todo!("Please choose a GPIO pin to use here, such as io.pins.gpio0!") as GpioPin<0>)    // Connect to LCK on PCM5102A DAC
        .build();

    let tx_buffer = i2s::take_tx_buffer();
    let mut transfer = i2s_tx.write_dma_circular_async(tx_buffer).unwrap();

    // GEN =============================
    let mut oscillator = SineOscillator::new(REFERENCE_FREQ);

    // Initialize a buffer to generate samples into before writing them to the DMA channel
    let mut buffer = i2s::new_chunk_buffer();
    let mut start = 0;
    loop {
        for sample in &mut buffer[start..] {
            let a = oscillator.generate();
            let b = (a * i16::MAX as f32) as i16 / 2;
            *sample = [b, b];
        }

        // W: written, S: skipped
        // [ W W W W W W W W W W W W W W W W S S S S ]
        //                                   ^ written
        let written = i2s::push(&mut transfer, &buffer).await;

        // [ S S S S _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ ]
        //           ^ start
        buffer.rotate_left(written);
        start = (buffer.len() - written) % buffer.len();
    }
}
