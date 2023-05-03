use super::Transform;
use crate::{
    iter_utils::{Cycle, CycleDirection},
    midi_event::{MIDIEvent, MIDIRouterEvent, NoteEvent, Wrap},
    scheduler::SchedulerHandler,
};
use schemars::JsonSchema;
use serde::Deserialize;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ArpeggioTransformOptions {
    subdivision: f64,
    direction: CycleDirection,
    repeat: Option<u64>,
    note_duration: Option<u64>,
}

#[derive(Debug)]
pub struct ArpeggioTransform {
    tempo_subdiv: Option<f64>,
    pressed_keys: Vec<NoteEvent>,
    note_duration: u64,
    cycle_iter: Cycle<NoteEvent>,
}

impl ArpeggioTransform {
    pub fn from_config(config: ArpeggioTransformOptions) -> ArpeggioTransform {
        ArpeggioTransform {
            tempo_subdiv: Some(config.subdivision),
            pressed_keys: vec![],
            note_duration: config.note_duration.unwrap_or(250),
            cycle_iter: Cycle::new(vec![], config.direction.clone(), config.repeat),
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

        let note_on = self.cycle_iter.next().clone();
        let note_off = note_on.get_note_off();

        scheduler.send_later(note_off.wrap(), self.note_duration);
        scheduler.send_now(note_on.wrap());
        None
    }

    fn on_message(
        &mut self,
        message: MIDIRouterEvent,
        _scheduler: &SchedulerHandler,
    ) -> Option<MIDIRouterEvent> {
        match message.event {
            MIDIEvent::NoteOff(NoteEvent { note, .. }) => {
                // Remove the current key by its note from the set of keys
                self.pressed_keys.retain(
                    |NoteEvent {
                         note: stored_note, ..
                     }| *stored_note != note,
                );
                self.cycle_iter.update_vec(self.pressed_keys.clone());

                None
            }

            MIDIEvent::NoteOn(note) => {
                self.pressed_keys.push(note);
                self.cycle_iter.update_vec(self.pressed_keys.clone());

                None
            }
            _ => Some(message),
        }
    }
}
