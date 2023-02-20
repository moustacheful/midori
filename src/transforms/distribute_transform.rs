use super::Transform;
use crate::{
    iter_utils::{Cycle, CycleDirection},
    midi_event::{MIDIEvent, MIDIRouterEvent, NoteEvent, Wrap},
    scheduler::SchedulerHandler,
};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct DistributeTransformOptions {
    between: Vec<u8>,
}

pub struct DistributeTransform {
    pressed_keys: Vec<NoteEvent>,
    between_iter: Cycle<u8>,
}

impl DistributeTransform {
    pub fn from_config(config: DistributeTransformOptions) -> Self {
        Self {
            between_iter: Cycle::new(config.between, CycleDirection::Forward, None),
            pressed_keys: vec![],
        }
    }
}

impl Transform for DistributeTransform {
    fn on_message(
        &mut self,
        mut message: MIDIRouterEvent,
        scheduler: &SchedulerHandler,
    ) -> Option<MIDIRouterEvent> {
        match message.event {
            MIDIEvent::NoteOn(ref mut note) => {
                note.channel = *self.between_iter.next();

                self.pressed_keys.push(note.clone());

                Some(message)
            }
            MIDIEvent::NoteOff(NoteEvent { note, .. }) => {
                // Remove all keys with this note
                self.pressed_keys.retain(|n| {
                    let should_keep = n.note != note;

                    // If this is to be removed, send a note off immediately
                    if !should_keep {
                        scheduler.send_now(n.get_note_off().wrap())
                    }

                    should_keep
                });

                None
            }
            _ => None,
        }
    }
}
