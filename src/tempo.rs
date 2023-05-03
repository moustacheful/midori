use async_trait::async_trait;
use futures::{Stream, StreamExt};
use std::future;
use std::time::Duration;
use tokio::sync::broadcast::{Receiver, Sender};
use tokio::time::Interval;
use tokio_stream::wrappers::BroadcastStream;

#[derive(Clone, Debug)]
pub enum ClockEvent {
    Tick,
    Restart,
}

pub struct ExternalClock {
    pub sender: Sender<ClockEvent>,
}

impl ExternalClock {
    pub fn new(ppqn: f64) -> (Self, ClockHandler) {
        let (sender, _) = tokio::sync::broadcast::channel::<ClockEvent>(999999);

        let sender_clone = sender.clone();

        (
            Self { sender },
            ClockHandler {
                ppqn,
                sender: sender_clone,
                bpm_sender: None,
            },
        )
    }
}

pub struct InternalClock {
    bpm: f64,
    // Pulses per quarter note (beat)
    ppqn: f64,
    interval: Interval,
    sender: Sender<ClockEvent>,
    bpm_receiver: flume::Receiver<f64>,
}

impl InternalClock {
    pub fn new(bpm: f64, ppqn: f64) -> (Self, ClockHandler) {
        let (sender, _) = tokio::sync::broadcast::channel::<ClockEvent>(999999);
        let (bpm_sender, bpm_receiver) = flume::unbounded::<f64>();
        let sender_clone = sender.clone();
        let interval = Self::get_interval(bpm, ppqn);

        (
            Self {
                bpm,
                ppqn,
                interval,
                sender,
                bpm_receiver,
            },
            ClockHandler {
                ppqn,
                sender: sender_clone,
                bpm_sender: Some(bpm_sender),
            },
        )
    }

    fn get_interval(bpm: f64, ppqn: f64) -> Interval {
        let beat_interval = (60.0 / bpm) * 1_000_000.0;
        let duration = Duration::from_micros((beat_interval / ppqn) as u64);

        tokio::time::interval(duration)
    }

    pub fn set_bpm(&mut self, bpm: f64) {
        self.bpm = bpm;

        self.interval = Self::get_interval(self.bpm, self.ppqn);
    }

    pub async fn start(mut self) {
        println!("Started clock");

        loop {
            tokio::select! {
                _ = self.interval.tick() => {
                    // Apparently having no receivers will result in an error
                    // so we do a preemptive check before forwarding this
                    if self.sender.receiver_count() > 0 {
                        self.sender.send(ClockEvent::Tick).unwrap();
                    }
                }

                Ok(new_bpm) = self.bpm_receiver.recv_async() => {
                    self.set_bpm(new_bpm);
                }
            }
        }
    }
}

#[derive(Clone)]
pub struct ClockHandler {
    ppqn: f64,
    sender: Sender<ClockEvent>,
    bpm_sender: Option<flume::Sender<f64>>,
}
impl ClockHandler {
    pub fn set_bpm(&self, bpm: f64) {
        if let Some(bpm_sender) = self.bpm_sender.as_ref() {
            bpm_sender.send(bpm).unwrap();
        }
    }

    pub fn create(&self, ratio: f64) -> impl Stream<Item = ()> {
        let receiver = self.sender.subscribe();
        every(
            BroadcastStream::new(receiver).map(|w| w.unwrap()),
            (self.ppqn * ratio) as u64,
        )
        .map(|_| ())
    }
}

pub fn every(s: impl Stream<Item = ClockEvent>, n: u64) -> impl Stream<Item = ClockEvent> {
    let mut count = 0;
    let max = n - 1;

    s.filter(move |evt| {
        let mut max_reached = count == max;

        match evt {
            ClockEvent::Tick => {
                count = if max_reached { 0 } else { count + 1 };
            }
            ClockEvent::Restart => {
                count = max;
                max_reached = true;
            }
        }

        future::ready(max_reached)
    })
}
