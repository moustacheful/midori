use futures::{Stream, StreamExt};
use std::time::Duration;

use crate::{app::MIDIMapperEvent, midi_event::MIDIRouterEvent};

#[derive(Debug, Clone)]
pub struct SchedulerHandler {
    pub sender: flume::Sender<MIDIRouterEvent>,
}

impl SchedulerHandler {
    pub fn send_now(&self, message: MIDIRouterEvent) {
        self.sender.send(message).unwrap();
    }

    pub fn send_later(&self, message: MIDIRouterEvent, delay_ms: u64) {
        let sender = self.sender.clone();

        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(delay_ms)).await;

            sender.send(message).unwrap();
        });
    }
}

pub struct Scheduler {
    pub receiver: flume::Receiver<MIDIRouterEvent>,
}

impl Scheduler {
    pub fn new() -> (Self, SchedulerHandler) {
        let (sender, receiver) = flume::unbounded::<MIDIRouterEvent>();

        (Self { receiver }, SchedulerHandler { sender })
    }

    pub fn stream(self) -> impl Stream<Item = MIDIMapperEvent> {
        self.receiver
            .into_stream()
            .map(MIDIMapperEvent::RouterMessage)
    }
}
