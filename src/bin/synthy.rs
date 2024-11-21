#![no_std]
#![no_main]

extern crate alloc;

use alloc::vec;
use embassy_executor::Spawner;
use embassy_futures::join::join4;
use embassy_sync::{blocking_mutex::raw::NoopRawMutex, mutex::Mutex};
use embassy_time::Duration;
use esp_backtrace as _;
use esp_hal::{
    dma::{Dma, DmaPriority},
    gpio::Io,
    i2s::{asynch::I2sWriteDmaAsync, I2sTx},
    timer::timg::TimerGroup,
};
use esp_println::println;
use synth::{
    i2s,
    input::{produce_midi_on_analog_input_change, AnalogInputBuilder, AnalogInputConfig},
    midi::{sequencer::produce_midi_for_note_sequence, MIDI_EVENTS},
    synth::Voice,
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
        .with_bclk(io.pins.gpio35)
        .with_dout(io.pins.gpio36)
        .with_ws(io.pins.gpio37)
        .build();

    let tx_buffer = i2s::take_tx_buffer();
    let mut transfer = i2s_tx.write_dma_circular_async(tx_buffer).unwrap();

    // ANALOG INPUTS ========================
    // This takes care of producing MIDI events when a potentiometer is turned
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

    // SEQUENCER ============================
    // A melody encoded as MIDI note keys.
    // The numbers 0 to 87 correspond the 88 keys of a piano
    let melody = vec![
        36, 39, 41, 43, 46, 48, 43, 39, 36, 34, 31, 29, 27, 31, 33, 36,
    ];
    // time between two successive "note on" events
    let beat_duration = Duration::from_millis(500);
    // time betwen a "note on" and following "note off" event
    let note_duration = Duration::from_millis(100);

    let seq_fut = produce_midi_for_note_sequence(&melody, beat_duration, note_duration);

    // GEN =============================
    // `synth` is a generator that will produce a new sample every time we make a call to
    // `.generate()`.
    // Because it is shared between multiple tasks, i.e. the generator task and midi event handling
    // task, we have to shield it from concurrent use with a mutex.
    let synth = Mutex::<NoopRawMutex, _>::new(Voice::new());

    // This task calls the `.handle_midi` method of `synth` when it receives a new event on
    // `MIDI_EVENTS`.
    let midi_fut = async {
        loop {
            let event = MIDI_EVENTS.receive().await;
            synth.lock().await.handle_midi(event);
        }
    };

    // This tasks does the most of the heavy lifting. It fills `buffer` with new samples by calling
    // `synth.generate()` and then pushes as many samples as possible to the i2s DMA.
    let gen_fut = async {
        // Initialize a buffer to generate samples into before writing them to the DMA channel
        let mut buffer = i2s::new_chunk_buffer();
        let mut start = 0;
        loop {
            for sample in &mut buffer[start..] {
                let mut synth = synth.lock().await;
                let a = synth.generate();
                let b = (a * i16::MAX as f32) as i16 / 2;
                *sample = [b, b];
                drop(synth);
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

    // All futures need to be awaited in order for the tasks to run.
    join4(midi_fut, gen_fut, analog_fut, seq_fut).await;
}
