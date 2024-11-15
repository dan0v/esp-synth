use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, channel::Channel};

pub mod send;
pub mod sequencer;
pub mod usb;

use midi_msg::MidiMsg;
pub use send::*;

pub static MIDI_EVENTS: Channel<CriticalSectionRawMutex, MidiMsg, 4> = Channel::new();
