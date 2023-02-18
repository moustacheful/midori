use super::Transform;
use crate::{
    midi_event::{MIDIEvent, MIDIRouterEvent, NoteEvent},
    scheduler::SchedulerHandler,
};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub enum ArpeggioDirection {
    Forward,
    Backwards,
    PingPong,
}

#[derive(Debug, Deserialize)]
pub struct ArpeggioTransformOptions {
    subdivision: f64,
    direction: ArpeggioDirection,
    repeat: u8,
    note_duration: Option<u64>,
}

#[derive(Debug)]
pub struct ArpeggioTransform {
    tempo_subdiv: Option<f64>,
    pressed_keys: Vec<NoteEvent>,
    current_index: usize,
    note_duration: u64,
}

impl ArpeggioTransform {
    pub fn from_config(config: ArpeggioTransformOptions) -> ArpeggioTransform {
        ArpeggioTransform {
            tempo_subdiv: Some(config.subdivision),
            pressed_keys: vec![],
            current_index: 0,
            note_duration: config.note_duration.unwrap_or(250),
        }
    }
}

impl Transform for ArpeggioTransform {
    fn get_tempo_subdiv(&self) -> Option<f64> {
        self.tempo_subdiv
    }

    fn on_tick(&mut self, scheduler: &SchedulerHandler) -> Option<MIDIRouterEvent> {
        if self.pressed_keys.is_empty() {
            return None;
        }

        let current_key = self
            .pressed_keys
            .get(self.current_index % self.pressed_keys.len());

        current_key?;

        let found = current_key.unwrap();
        let note_off = found.get_note_off();

        self.current_index += 1;

        let next_message = MIDIRouterEvent {
            device: "self".to_string(),
            event: MIDIEvent::NoteOn(found.to_owned()),
        };

        scheduler.send_later(
            MIDIRouterEvent {
                device: "self".to_string(),
                event: MIDIEvent::NoteOff(note_off),
            },
            self.note_duration,
        );

        Some(next_message)
    }

    fn on_message(
        &mut self,
        message: MIDIRouterEvent,
        _scheduler: &SchedulerHandler,
    ) -> Option<MIDIRouterEvent> {
        match message.event {
            MIDIEvent::NoteOff(note) => {
                self.pressed_keys.retain(|stored_note| *stored_note != note);
                // self.current_index = 0;
                None
            }
            MIDIEvent::NoteOn(note) => {
                self.pressed_keys.push(note);
                self.current_index = 0;
                None
            }
            _ => Some(message),
        }
    }
}
