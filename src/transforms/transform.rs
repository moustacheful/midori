use crate::{midi_mapper::MidiRouterMessage, MidiRouterMessageWrapper};

pub trait Transform {
    fn get_tempo_subdiv(&self) -> Option<u64> {
        None
    }

    // This triggers on what we subscribe as points of interest, e.g. an arpeggio?
    fn on_tick(&mut self) -> Option<MidiRouterMessage> {
        None
    }

    fn on_message(&mut self, v: MidiRouterMessage) -> Option<MidiRouterMessage> {
        Some(v)
    }

    fn process_message(&mut self, message: MidiRouterMessageWrapper) -> Option<MidiRouterMessage> {
        match message {
            MidiRouterMessageWrapper::Tick => self.on_tick(),
            MidiRouterMessageWrapper::RouterMessage(message) => self.on_message(message),
        }
    }
}
