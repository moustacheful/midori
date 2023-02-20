use super::Transform;
use crate::{
    iter_utils::{Cycle, CycleDirection},
    midi_event::{MIDIEvent, MIDIRouterEvent},
    scheduler::SchedulerHandler,
};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct DistributeTransformOptions {
    between: Vec<u8>,
}

pub struct DistributeTransform {
    between_iter: Cycle<u8>,
}

impl DistributeTransform {
    pub fn from_config(config: DistributeTransformOptions) -> Self {
        Self {
            between_iter: Cycle::new(config.between, CycleDirection::Forward, None),
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
                note.channel = *self.between_iter.next();

                Some(message)
            }
            _ => None,
        }
    }
}
