use serde::Deserialize;

use crate::{app::MidiRouterMessageWrapper, midi_mapper::MidiRouterMessage};

use super::{
    arpeggio_transform::ArpeggioTransformOptions, FilterTransformOptions, MapTransformOptions,
};

#[derive(Debug, Deserialize)]
pub enum SerializedTransform {
    Filter(FilterTransformOptions),
    Arpeggio(ArpeggioTransformOptions),
    Map(MapTransformOptions),
    Output(String),
}

pub trait Transform {
    fn get_tempo_subdiv(&self) -> Option<u64> {
        None
    }

    // This triggers on what we subscribe as points of interest, e.g. an arpeggio?
    fn on_tick(&mut self) -> Option<MidiRouterMessage> {
        None
    }

    fn on_message(&mut self, message: MidiRouterMessage) -> Option<MidiRouterMessage> {
        Some(message)
    }

    fn process_message(&mut self, message: MidiRouterMessageWrapper) -> Option<MidiRouterMessage> {
        match message {
            MidiRouterMessageWrapper::Tick => self.on_tick(),
            MidiRouterMessageWrapper::RouterMessage(message) => self.on_message(message),
        }
    }
}
