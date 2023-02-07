use futures::{Stream, StreamExt};
use std::time::Duration;

use crate::{app::MidiRouterMessageWrapper, midi_mapper::MidiRouterMessage};

#[derive(Debug, Clone)]
pub struct SchedulerHandler {
    pub sender: flume::Sender<MidiRouterMessage>,
}

impl SchedulerHandler {
    pub fn send_now(&self, message: MidiRouterMessage) {
        self.sender.send(message).unwrap();
    }

    pub fn send_later(&self, message: MidiRouterMessage, delay_ms: u64) {
        let sender = self.sender.clone();

        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(delay_ms)).await;

            sender.send(message).unwrap();
        });
    }
}

pub struct Scheduler {
    pub receiver: flume::Receiver<MidiRouterMessage>,
}

impl Scheduler {
    pub fn new() -> (Self, SchedulerHandler) {
        let (sender, receiver) = flume::unbounded::<MidiRouterMessage>();

        (Self { receiver }, SchedulerHandler { sender })
    }

    pub fn stream(self) -> impl Stream<Item = MidiRouterMessageWrapper> {
        self.receiver.clone().into_stream().map(|m| {
            println!("sending noteoff!");
            MidiRouterMessageWrapper::RouterMessage(m)
        })
    }
}
