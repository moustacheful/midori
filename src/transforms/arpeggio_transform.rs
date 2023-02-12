use super::Transform;
use crate::{midi_event::MidiEvent, midi_mapper::MidiRouterMessage, scheduler::SchedulerHandler};
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
    pressed_keys: Vec<MidiEvent>,
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

    fn on_tick(&mut self, scheduler: &SchedulerHandler) -> Option<MidiRouterMessage> {
        if self.pressed_keys.is_empty() {
            return None;
        }

        let current_key = self
            .pressed_keys
            .get(self.current_index % self.pressed_keys.len());

        current_key?;

        let found = current_key.unwrap();
        self.current_index += 1;

        let next_message = MidiRouterMessage {
            device: "self".to_string(),
            event: *found,
        };

        let next_note_off = next_message.event.get_note_off();

        if let Some(event) = next_note_off {
            scheduler.send_later(
                MidiRouterMessage {
                    device: "self".to_string(),
                    event,
                },
                self.note_duration,
            )
        }

        Some(next_message)
    }

    fn on_message(
        &mut self,
        message: MidiRouterMessage,
        _scheduler: &SchedulerHandler,
    ) -> Option<MidiRouterMessage> {
        match message.event {
            MidiEvent::NoteOff { note, .. } => {
                self.pressed_keys.retain(|v| match v {
                    MidiEvent::NoteOn {
                        note: stored_note, ..
                    } => *stored_note != note,
                    _ => true,
                });
                // self.current_index = 0;
                None
            }
            MidiEvent::NoteOn { .. } => {
                self.pressed_keys.push(message.event);
                self.current_index = 0;
                None
            }
            _ => Some(message),
        }
    }
}
