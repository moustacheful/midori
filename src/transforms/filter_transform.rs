use super::Transform;
use crate::{
    midi_event::{MIDIEventIdentity, MIDIRouterEvent},
    scheduler::SchedulerHandler,
};
use schemars::JsonSchema;
use serde::Deserialize;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct FilterTransformOptions {
    pub devices: Option<Vec<String>>,
    pub channels: Option<Vec<u8>>,
    pub event_types: Option<Vec<MIDIEventIdentity>>,
}

pub struct FilterTransform {
    devices: Vec<String>,
    channels: Vec<u8>,
    event_types: Vec<MIDIEventIdentity>,
}

impl FilterTransform {
    pub fn from_config(options: FilterTransformOptions) -> Self {
        Self {
            devices: options.devices.unwrap_or(vec![]),
            channels: options.channels.unwrap_or(vec![]),
            event_types: options.event_types.unwrap_or(vec![]),
        }
    }
}

impl FilterTransform {
    fn should_pass(&self, midi_router_event: MIDIRouterEvent) -> Option<MIDIRouterEvent> {
        let checks: Vec<fn(&Self, &MIDIRouterEvent) -> bool> = vec![
            |s, e| {
                if s.devices.is_empty() {
                    return true;
                }

                s.devices.contains(&e.device)
            },
            |s, e| {
                if s.event_types.is_empty() {
                    return true;
                }

                s.event_types.contains(&e.event.get_identity())
            },
            |s, e| {
                if s.channels.is_empty() {
                    return true;
                }

                s.channels.contains(&e.event.get_channel())
            },
        ];

        // If any of the checks return false, this will stop iterating
        let disallowed = checks.iter().any(|check| !check(self, &midi_router_event));

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
