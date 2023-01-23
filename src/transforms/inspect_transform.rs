use super::Transform;
use crate::{midi_mapper::MidiRouterMessage, MidiRouterMessageWrapper, Wrapper};

pub struct InspectTransform {
    pub prefix: String,
}

impl Transform for InspectTransform {
    fn on_message(&mut self, v: MidiRouterMessage) -> Option<MidiRouterMessage> {
        println!("[{:?}]: {:?}", self.prefix, v);

        Some(v)
    }
}
