use std::{iter::Cycle, vec::IntoIter};

use serde::Deserialize;

use super::Transform;
use crate::{
    midi_event::{MIDIEvent, MIDIRouterEvent},
    scheduler::SchedulerHandler,
};

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
        mut message: MIDIRouterEvent,
        _scheduler: &SchedulerHandler,
    ) -> Option<MIDIRouterEvent> {
        match message.event {
            MIDIEvent::NoteOn(ref mut note) => {
                note.channel = self.between_iter.next().unwrap();

                Some(message)
            }
            _ => None,
        }
    }
}
