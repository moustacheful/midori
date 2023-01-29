use super::Transform;
use crate::midi_mapper::MidiRouterMessage;

pub struct OutputTransform {
    output_device: String,
}

impl OutputTransform {
    pub fn from_config(output_device: String) -> Self {
        Self { output_device }
    }
}

impl Transform for OutputTransform {
    fn on_message(&mut self, mut v: MidiRouterMessage) -> Option<MidiRouterMessage> {
        v.device = self.output_device.clone();

        Some(v)
    }
}
