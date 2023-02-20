use super::Transform;
use crate::{
    midi_event::{MIDIEvent, MIDIEventIdentity, MIDIRouterEvent},
    scheduler::SchedulerHandler,
};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct FilterTransformOptions {
    pub channels: Option<Vec<u8>>,
    pub event_types: Option<Vec<MIDIEventIdentity>>,
}

pub struct FilterTransform {
    channels: Vec<u8>,
    event_types: Vec<MIDIEventIdentity>,
}

impl FilterTransform {
    pub fn from_config(options: FilterTransformOptions) -> Self {
        Self {
            channels: options.channels.unwrap_or(vec![]),
            event_types: options.event_types.unwrap_or(vec![]),
        }
    }
}

impl FilterTransform {
    fn should_pass(&self, midi_router_event: MIDIRouterEvent) -> Option<MIDIRouterEvent> {
        let event = &midi_router_event.event;
        let checks: Vec<fn(&Self, &MIDIEvent) -> bool> = vec![
            |s, e| {
                if s.event_types.is_empty() {
                    return true;
                }

                s.event_types.contains(&e.get_identity())
            },
            |s, e| {
                if s.channels.is_empty() {
                    return true;
                }

                s.channels.contains(&e.get_channel())
            },
        ];

        // If any of the checks return false, this will stop iterating
        let disallowed = checks.iter().any(|check| !check(self, event));

        if disallowed {
            None
        } else {
            Some(midi_router_event)
        }
    }
}

impl Transform for FilterTransform {
    fn on_message(
        &mut self,
        message: MIDIRouterEvent,
        _scheduler: &SchedulerHandler,
    ) -> Option<MIDIRouterEvent> {
        self.should_pass(message)
    }
}
