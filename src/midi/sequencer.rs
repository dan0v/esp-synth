use alloc::boxed::Box;
use embassy_time::{Duration, Ticker, Timer};

use super::{send_note_off, send_note_on};

/// Produce NoteOn and NoteOff events for each note in the sequence
#[embassy_executor::task]
pub async fn sequencer(melody: Box<[u8]>, beat_duration: Duration, note_duration: Duration) {
    let mut beat = Ticker::every(beat_duration);
    for note in melody.iter().cycle() {
        beat.next().await;
        let note_off = Timer::after(note_duration);
        send_note_on(*note, 127).await;

        note_off.await;
        send_note_off(*note, 127).await;
    }
}
