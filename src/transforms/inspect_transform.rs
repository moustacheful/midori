use super::Transform;
use crate::{midi_event::MIDIRouterEvent, scheduler::SchedulerHandler};

pub struct InspectTransform {
    pub prefix: String,
}

impl Transform for InspectTransform {
    fn on_message(
        &mut self,
        v: MIDIRouterEvent,
        _scheduler: &SchedulerHandler,
    ) -> Option<MIDIRouterEvent> {
        println!("[{:?}]: {:?}", self.prefix, v);

        Some(v)
    }
}
