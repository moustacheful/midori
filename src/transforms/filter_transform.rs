use super::Transform;
use crate::midi_mapper::MidiRouterMessage;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct FilterTransformOptions {
    pub channels: Vec<u8>,
}

pub struct FilterTransform {
    channels: Vec<u8>,
}

impl FilterTransform {
    pub fn from_config(options: FilterTransformOptions) -> Self {
        Self {
            channels: options.channels,
        }
    }
}

impl Transform for FilterTransform {
    fn on_message(&mut self, message: MidiRouterMessage) -> Option<MidiRouterMessage> {
        let message_channel = message.event.get_channel();

        if self.channels.contains(message_channel) {
            return Some(message);
        }

        None
    }
}
