use std::collections::HashMap;

use super::Transform;
use crate::{midi_event::MidiEvent, midi_mapper::MidiRouterMessage};

pub struct MapTransformOptions {
    pub channels: Vec<(u8, u8)>,
    pub cc: Vec<(u8, u8)>,
}

pub struct MapTransform {
    channels: HashMap<u8, u8>,
    cc: HashMap<u8, u8>,
}

impl MapTransform {
    pub fn new(options: MapTransformOptions) -> Self {
        Self {
            channels: HashMap::from_iter(options.channels),
            cc: HashMap::from_iter(options.cc),
        }
    }
}

impl Transform for MapTransform {
    fn on_message(&mut self, mut message: MidiRouterMessage) -> Option<MidiRouterMessage> {
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
