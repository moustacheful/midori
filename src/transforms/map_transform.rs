use std::collections::HashMap;

use schemars::JsonSchema;
use serde::Deserialize;

use super::Transform;
use crate::{
    midi_event::{MIDIEvent, MIDIRouterEvent},
    scheduler::SchedulerHandler,
};

#[derive(Debug, Deserialize, JsonSchema)]
pub struct MapTransformOptions {
    pub channels: Option<Vec<(u8, u8)>>,
    pub cc: Option<Vec<(u8, u8)>>,
}

pub struct MapTransform {
    channels: HashMap<u8, u8>,
    cc: HashMap<u8, u8>,
}

impl MapTransform {
    pub fn from_config(options: MapTransformOptions) -> Self {
        Self {
            channels: HashMap::from_iter(options.channels.unwrap_or_default()),
            cc: HashMap::from_iter(options.cc.unwrap_or_default()),
        }
    }
}

impl Transform for MapTransform {
    fn on_message(
        &mut self,
        mut message: MIDIRouterEvent,
        _scheduler: &SchedulerHandler,
    ) -> Option<MIDIRouterEvent> {
        let current_channel = message.event.get_channel();

        // Map channel
        if let Some(target_channel) = self.channels.get(&current_channel) {
            message.event.set_channel(*target_channel);
        }

        // Map CCs
        if let MIDIEvent::Controller(ref mut controller) = message.event {
            if let Some(target_cc) = self.cc.get(&controller.controller) {
                controller.controller = *target_cc;
            }
        }

        Some(message)
    }
}
