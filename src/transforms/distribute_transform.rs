use std::{iter::Cycle, vec::IntoIter};

use serde::Deserialize;

use super::Transform;
use crate::{midi_event::MidiEvent, midi_mapper::MidiRouterMessage, scheduler::SchedulerHandler};

#[derive(Debug, Deserialize)]
pub struct DistributeTransformOptions {
    between: Vec<u8>,
}

pub struct DistributeTransform {
    between_iter: Cycle<IntoIter<u8>>,
}

impl DistributeTransform {
    pub fn from_config(config: DistributeTransformOptions) -> Self {
        Self {
            between_iter: config.between.into_iter().cycle(),
        }
    }
}

impl Transform for DistributeTransform {
    fn on_message(
        &mut self,
        mut message: MidiRouterMessage,
        scheduler: &SchedulerHandler,
    ) -> Option<MidiRouterMessage> {
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
