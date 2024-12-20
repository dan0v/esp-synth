#![no_std]
#![no_main]

extern crate alloc;

use alloc::vec;
use embassy_executor::Spawner;
use embassy_futures::join::join3;
use embassy_sync::{blocking_mutex::raw::NoopRawMutex, mutex::Mutex};
use embassy_time::Duration;
use esp_backtrace as _;
use esp_hal::{
    cpu_control::{CpuControl, Stack},
    dma::{Dma, DmaPriority},
    gpio::{Io, Level, Output},
    i2s::{asynch::I2sWriteDmaAsync, I2sTx},
    otg_fs::Usb,
    timer::timg::TimerGroup,
};
use esp_hal_embassy::Executor;
use esp_println::println;
use static_cell::StaticCell;
use synth::{
    i2s,
    input::{produce_midi_on_analog_input_change, AnalogInputBuilder, AnalogInputConfig},
    midi::{sequencer::sequencer, usb::handle_usb, MIDI_EVENTS},
    voice::Voice,
};

static APP_CORE_STACK: StaticCell<Stack<8192>> = StaticCell::new();

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
        .with_bclk(io.pins.gpio35)
        .with_dout(io.pins.gpio36)
        .with_ws(io.pins.gpio37)
        .build();

    let tx_buffer = i2s::take_tx_buffer();
    let mut transfer = i2s_tx.write_dma_circular_async(tx_buffer).unwrap();

    // USB MIDI =============================
    // Define the USB peripheral and the D+ and D- pins
    // GPIO19 and GPIO20 are connected to the second USB-C connector
    let usb = Usb::new(peripherals.USB0, io.pins.gpio20, io.pins.gpio19);

    // ANALOG INPUTS ========================
    let analog_input_config = AnalogInputConfig {
        alpha: 0.8,
        trigger_threshold: 16,
        sustain_threshold: 8,
    };
    let (mut adc, mut analog_inputs) = AnalogInputBuilder::new(analog_input_config)
        .add(io.pins.gpio7, 14)
        .add(io.pins.gpio6, 15)
        .add(io.pins.gpio5, 17)
        .add(io.pins.gpio4, 23)
        .build(peripherals.ADC1);

    let analog_fut = produce_midi_on_analog_input_change(
        &mut adc,
        &mut analog_inputs,
        Duration::from_millis(10),
    );

    // Spin up the second (APP) core with the `handle_usb` task
    let mut cpu_control = CpuControl::new(peripherals.CPU_CTRL);
    let _guard = cpu_control
        .start_app_core(APP_CORE_STACK.init(Stack::new()), move || {
            static EXECUTOR: StaticCell<Executor> = StaticCell::new();
            let executor = EXECUTOR.init(Executor::new());
            executor.run(|spawner| {
                spawner.spawn(handle_usb(usb)).ok();
            });
        })
        .unwrap();

    // GEN =============================
    let voice = Mutex::<NoopRawMutex, _>::new(Voice::new());

    let midi_fut = async {
        loop {
            let event = MIDI_EVENTS.receive().await;
            voice.lock().await.handle_midi(event);
        }
    };

    let gen_fut = async {
        // Initialize a buffer to generate samples into before writing them to the DMA channel
        let mut buffer = i2s::new_chunk_buffer();
        let mut start = 0;
        loop {
            for sample in &mut buffer[start..] {
                let mut voice = voice.lock().await;
                let a = voice.generate();
                let b = (a * i16::MAX as f32) as i16 / 2;
                *sample = [b, b];
                drop(voice);
            }

            // W: written, S: skipped
            // [ W W W W W W W W W W W W W W W W S S S S ]
            //                                   ^ written
            let written = i2s::push(&mut transfer, &buffer).await;

            // [ S S S S _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ ]
            //           ^ start
            buffer.rotate_left(written);
            start = buffer.len() - written;
        }
    };

    join3(midi_fut, gen_fut, analog_fut).await;
}
