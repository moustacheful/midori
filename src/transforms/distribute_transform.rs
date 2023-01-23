use std::{iter::Cycle, vec::IntoIter};

use super::Transform;
use crate::{midi_event::MidiEvent, midi_mapper::MidiRouterMessage};

pub struct DistributeTransform {
    between_iter: Cycle<IntoIter<u8>>,
}

impl DistributeTransform {
    pub fn new(between: Vec<u8>) -> Self {
        Self {
            between_iter: between.into_iter().cycle(),
        }
    }
}

impl Transform for DistributeTransform {
    fn on_message(&mut self, mut message: MidiRouterMessage) -> Option<MidiRouterMessage> {
        match message.event {
            MidiEvent::NoteOn { note, velocity, .. } => {
                message.event = MidiEvent::NoteOn {
                    note,
                    velocity,
                    channel: self.between_iter.next().unwrap(),
                };
                Some(message)
            }
            _ => None,
        }
    }
}
