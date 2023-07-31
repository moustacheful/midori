use std::collections::HashMap;

use schemars::JsonSchema;
use serde::Deserialize;

use super::Transform;
use crate::{midi_event::MIDIRouterEvent, scheduler::SchedulerHandler};

#[derive(Debug, Deserialize, JsonSchema)]
pub struct MirrorTransformOptions {
    pub channels: Option<Vec<u8>>,
}

pub struct MirrorTransform {
    channels: Vec<u8>,
}

impl MirrorTransform {
    pub fn from_config(options: MirrorTransformOptions) -> Self {
        Self {
            channels: options.channels.unwrap_or_default(),
        }
    }
}

impl Transform for MirrorTransform {
    fn on_message(
        &mut self,
        message: MIDIRouterEvent,
        scheduler: &SchedulerHandler,
    ) -> Option<MIDIRouterEvent> {
        self.channels.iter().for_each(|channel| {
            let mut current = message.clone();
            current.event.set_channel(channel.clone());
            scheduler.send_now(current)
        });

        None
    }
}
