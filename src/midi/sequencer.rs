use alloc::vec::Vec;
use embassy_time::{Duration, Ticker, Timer};

use super::{send_note_off, send_note_on};

#[embassy_executor::task]
pub async fn sequencer(melody: Vec<u8>, beat_duration: Duration, note_duration: Duration) {
    produce_midi_for_note_sequence(&melody, beat_duration, note_duration).await;
}

/// Produce NoteOn and NoteOff events for each note in the sequence
pub async fn produce_midi_for_note_sequence(
    melody: &[u8],
    beat_duration: Duration,
    note_duration: Duration,
) {
    let mut beat = Ticker::every(beat_duration);
    for note in melody.iter().cycle() {
        beat.next().await;
        let note_off = Timer::after(note_duration);
        send_note_on(*note, 127).await;

        note_off.await;
        send_note_off(*note, 127).await;
    }
}
