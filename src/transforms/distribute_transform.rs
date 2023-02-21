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

#[cfg(test)]
mod tests {
    use crate::{
        app::MIDIMapperEvent,
        midi_event::{NoteEvent, Wrap},
        scheduler::Scheduler,
        transforms::Transform,
    };

    use super::{DistributeTransform, DistributeTransformOptions};

    fn get_transform_instance() -> DistributeTransform {
        let config = DistributeTransformOptions {
            between: vec![9, 2, 4],
        };

        DistributeTransform::from_config(config)
    }

    #[test]
    fn distributes_between_channels() {
        let mut transform = get_transform_instance();
        let (_scheduler, scheduler_handler) = Scheduler::new();

        let note_on = NoteEvent {
            channel: 3,
            note: 1,
            velocity: 127,
        }
        .wrap();

        let result: Vec<_> = vec![
            transform.process_message(
                MIDIMapperEvent::RouterMessage(note_on.clone()),
                &scheduler_handler,
            ),
            transform.process_message(
                MIDIMapperEvent::RouterMessage(note_on.clone()),
                &scheduler_handler,
            ),
            transform.process_message(
                MIDIMapperEvent::RouterMessage(note_on.clone()),
                &scheduler_handler,
            ),
        ]
        .into_iter()
        .map(|msg| {
            let router_event = msg.unwrap();

            router_event.event.get_channel()
        })
        .collect();

        assert_eq!(result, vec![9, 2, 4])
    }
}
