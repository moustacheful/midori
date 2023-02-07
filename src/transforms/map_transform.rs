use std::collections::HashMap;

use serde::Deserialize;

use super::Transform;
use crate::{midi_event::MidiEvent, midi_mapper::MidiRouterMessage, scheduler::SchedulerHandler};

#[derive(Debug, Deserialize)]
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
            channels: HashMap::from_iter(options.channels.unwrap_or(vec![])),
            cc: HashMap::from_iter(options.cc.unwrap_or(vec![])),
        }
    }
}

impl Transform for MapTransform {
    fn on_message(
        &mut self,
        mut message: MidiRouterMessage,
        scheduler: &SchedulerHandler,
    ) -> Option<MidiRouterMessage> {
        let current_channel = message.event.get_channel();

        // Map channel
        if let Some(target_channel) = self.channels.get(current_channel) {
            message.event = message.event.set_channel(*target_channel);
        }

        // Map CCs
        if let MidiEvent::Controller {
            controller,
            channel,
            value,
        } = message.event
        {
            if let Some(target_cc) = self.cc.get(&controller) {
                message.event = MidiEvent::Controller {
                    controller: *target_cc,
                    channel,
                    value,
                }
            }
        }

        Some(message)
    }
}
