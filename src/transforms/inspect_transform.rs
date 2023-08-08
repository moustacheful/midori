use schemars::JsonSchema;
use serde::Deserialize;

use super::Transform;
use crate::{midi_event::MIDIRouterEvent, scheduler::SchedulerHandler};

#[derive(Debug, Deserialize, JsonSchema)]
pub struct InspectTransformOptions {
    pub prefix: Option<String>,
}

// A helper transform that prints whatever it receives, with an optional prefix
pub struct InspectTransform {
    pub prefix: String,
}

impl InspectTransform {
    pub fn from_config(options: InspectTransformOptions) -> Self {
        Self {
            prefix: options
                .prefix
                .map(|prefix| format!("[{prefix}]"))
                .unwrap_or("".into()),
        }
    }
}

impl Transform for InspectTransform {
    fn on_message(
        &mut self,
        v: MIDIRouterEvent,
        _scheduler: &SchedulerHandler,
    ) -> Option<MIDIRouterEvent> {
        println!("{}{}", self.prefix, v);

        Some(v)
    }
}
