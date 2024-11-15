use midi_msg::{Channel::Ch1, ControlChange, MidiMsg};

use super::MIDI_EVENTS;

pub async fn send_control(control: u8, value: u8) {
    let msg = MidiMsg::ChannelVoice {
        channel: Ch1,
        msg: midi_msg::ChannelVoiceMsg::ControlChange {
            control: ControlChange::CC { control, value },
        },
    };
    MIDI_EVENTS.send(msg).await;
}

pub async fn send_note_on(note: u8, velocity: u8) {
    let msg = MidiMsg::ChannelVoice {
        channel: Ch1,
        msg: midi_msg::ChannelVoiceMsg::NoteOn { note, velocity },
    };
    MIDI_EVENTS.send(msg).await;
}

pub async fn send_note_off(note: u8, velocity: u8) {
    let msg = MidiMsg::ChannelVoice {
        channel: Ch1,
        msg: midi_msg::ChannelVoiceMsg::NoteOff { note, velocity },
    };
    MIDI_EVENTS.send(msg).await;
}
