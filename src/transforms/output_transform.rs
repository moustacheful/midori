use schemars::JsonSchema;
use serde::Deserialize;

use super::Transform;
use crate::{midi_event::MIDIRouterEvent, scheduler::SchedulerHandler};

#[derive(Debug, Deserialize, JsonSchema)]
pub struct OutputTransformOptions {
    pub output_device: String,
}

pub struct OutputTransform {
    output_device: String,
}

impl OutputTransform {
    pub fn from_config(options: OutputTransformOptions) -> Self {
        Self {
            output_device: options.output_device,
        }
    }
}

impl Transform for OutputTransform {
    fn on_message(
        &mut self,
        mut v: MIDIRouterEvent,
        _scheduler: &SchedulerHandler,
    ) -> Option<MIDIRouterEvent> {
        v.device = self.output_device.clone();

        Some(v)
    }
}
