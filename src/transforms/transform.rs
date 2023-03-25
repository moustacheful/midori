use serde::Deserialize;

use crate::{app::MIDIMapperEvent, midi_event::MIDIRouterEvent, scheduler::SchedulerHandler};

use super::{
    arpeggio_transform::ArpeggioTransformOptions, distribute_transform::DistributeTransformOptions,
    wasm_transform::WasmTransformOptions, FilterTransformOptions, MapTransformOptions,
};

#[derive(Debug, Deserialize)]
pub enum SerializedTransform {
    Filter(FilterTransformOptions),
    Arpeggio(ArpeggioTransformOptions),
    Map(MapTransformOptions),
    Distribute(DistributeTransformOptions),
    Wasm(WasmTransformOptions),
    Output(String),
    Inspect(String),
}

pub trait Transform {
    fn set_scheduler(&mut self, _scheduler: SchedulerHandler) {}

    fn get_tempo_subdiv(&self) -> Option<f64> {
        None
    }

    // This triggers on what we subscribe as points of interest, e.g. an arpeggio?
    fn on_tick(&mut self, _scheduler: &SchedulerHandler) -> Option<MIDIRouterEvent> {
        None
    }

    fn on_message(
        &mut self,
        message: MIDIRouterEvent,
        _scheduler: &SchedulerHandler,
    ) -> Option<MIDIRouterEvent> {
        Some(message)
    }

    fn process_message(
        &mut self,
        message: MIDIMapperEvent,
        scheduler: &SchedulerHandler,
    ) -> Option<MIDIRouterEvent> {
        match message {
            MIDIMapperEvent::Tick => self.on_tick(scheduler),
            MIDIMapperEvent::RouterMessage(message) => self.on_message(message, scheduler),
        }
    }
}
