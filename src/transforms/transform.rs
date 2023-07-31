use schemars::JsonSchema;
use serde::Deserialize;

use crate::{app::MIDIMapperEvent, midi_event::MIDIRouterEvent, scheduler::SchedulerHandler};

use super::{
    arpeggio_transform::ArpeggioTransformOptions, distribute_transform::DistributeTransformOptions,
    inspect_transform::InspectTransformOptions, mirror_transform::MirrorTransformOptions,
    output_transform::OutputTransformOptions, wasm_transform::WasmTransformOptions,
    FilterTransformOptions, MapTransformOptions,
};

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(tag = "type")]
pub enum SerializedTransform {
    Filter(FilterTransformOptions),
    Arpeggio(ArpeggioTransformOptions),
    Map(MapTransformOptions),
    Distribute(DistributeTransformOptions),
    Wasm(WasmTransformOptions),
    Output(OutputTransformOptions),
    Inspect(InspectTransformOptions),
    Mirror(MirrorTransformOptions),
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
