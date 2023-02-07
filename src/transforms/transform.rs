use serde::Deserialize;

use crate::{
    app::MidiRouterMessageWrapper, midi_mapper::MidiRouterMessage, scheduler::SchedulerHandler,
};

use super::{
    arpeggio_transform::ArpeggioTransformOptions, distribute_transform::DistributeTransformOptions,
    FilterTransformOptions, MapTransformOptions,
};

#[derive(Debug, Deserialize)]
pub enum SerializedTransform {
    Filter(FilterTransformOptions),
    Arpeggio(ArpeggioTransformOptions),
    Map(MapTransformOptions),
    Distribute(DistributeTransformOptions),
    Output(String),
    Inspect(String),
}

pub trait Transform {
    fn get_tempo_subdiv(&self) -> Option<f64> {
        None
    }

    // This triggers on what we subscribe as points of interest, e.g. an arpeggio?
    fn on_tick(&mut self, scheduler: &SchedulerHandler) -> Option<MidiRouterMessage> {
        None
    }

    fn on_message(
        &mut self,
        message: MidiRouterMessage,
        scheduler: &SchedulerHandler,
    ) -> Option<MidiRouterMessage> {
        Some(message)
    }

    fn process_message(
        &mut self,
        message: MidiRouterMessageWrapper,
        scheduler: &SchedulerHandler,
    ) -> Option<MidiRouterMessage> {
        match message {
            MidiRouterMessageWrapper::Tick => self.on_tick(scheduler),
            MidiRouterMessageWrapper::RouterMessage(message) => self.on_message(message, scheduler),
        }
    }
}
