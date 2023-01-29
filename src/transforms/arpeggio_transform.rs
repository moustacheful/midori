use serde::Deserialize;

use super::Transform;
use crate::{midi_event::MidiEvent, midi_mapper::MidiRouterMessage};

#[derive(Debug, Deserialize)]
pub enum ArpeggioDirection {
    Forward,
    Backwards,
    PingPong,
}

#[derive(Debug, Deserialize)]
pub struct ArpeggioTransformOptions {
    subdivision: u64,
    direction: ArpeggioDirection,
    repeat: u8,
}

#[derive(Debug)]
pub struct ArpeggioTransform {
    tempo_subdiv: Option<u64>,
    pressed_keys: Vec<MidiEvent>,
    current_index: usize,
}

impl ArpeggioTransform {
    pub fn from_config(config: ArpeggioTransformOptions) -> ArpeggioTransform {
        ArpeggioTransform {
            tempo_subdiv: Some(config.subdivision),
            pressed_keys: vec![],
            current_index: 0,
        }
    }

    pub fn new(tempo_subdiv: Option<u64>) -> ArpeggioTransform {
        ArpeggioTransform {
            tempo_subdiv,
            pressed_keys: vec![],
            current_index: 0,
        }
    }
}

impl Transform for ArpeggioTransform {
    fn get_tempo_subdiv(&self) -> Option<u64> {
        self.tempo_subdiv
    }

    fn on_tick(&mut self) -> Option<MidiRouterMessage> {
        if self.pressed_keys.len() == 0 {
            return None;
        }

        let current_key = self
            .pressed_keys
            .get(self.current_index % self.pressed_keys.len());

        if let Some(found) = current_key {
            self.current_index += 1;

            return Some(MidiRouterMessage {
                device: "self".to_string(),
                event: found.clone(),
            });
        }

        None
    }

    fn on_message(&mut self, message: MidiRouterMessage) -> Option<MidiRouterMessage> {
        match message.event {
            MidiEvent::NoteOff { note, .. } => {
                self.pressed_keys.retain(|v| match v {
                    MidiEvent::NoteOn {
                        note: stored_note, ..
                    } => *stored_note != note,
                    _ => true,
                });
                // self.current_index = 0;
                return None;
            }
            MidiEvent::NoteOn { .. } => {
                self.pressed_keys.push(message.event);
                self.current_index = 0;
                return None;
            }
            _ => return Some(message),
        }
    }
}
