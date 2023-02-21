use futures::{Stream, StreamExt};
use std::future;
use std::time::Duration;
use tokio::sync::broadcast::Sender;
use tokio::time::Interval;
use tokio_stream::wrappers::BroadcastStream;

pub struct Clock {
    bpm: f64,
    // Pulses per quarter note (beat)
    ppqn: f64,
    interval: Interval,
    sender: Sender<()>,
    bpm_receiver: flume::Receiver<f64>,
}

impl Clock {
    pub fn new(bpm: f64, ppqn: f64) -> (Self, ClockHandler) {
        let (sender, _) = tokio::sync::broadcast::channel::<()>(999999);
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
                bpm_sender,
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
                        self.sender.send(()).unwrap();
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
    sender: Sender<()>,
    bpm_sender: flume::Sender<f64>,
}
impl ClockHandler {
    pub fn set_bpm(&self, bpm: f64) {
        self.bpm_sender.send(bpm).unwrap();
    }

    pub fn create(&self, ratio: f64) -> impl Stream<Item = ()> {
        let receiver = self.sender.subscribe();
        every(BroadcastStream::new(receiver), (self.ppqn * ratio) as u64).map(|w| w.unwrap())
    }
}

pub fn every<I>(s: impl Stream<Item = I>, n: u64) -> impl Stream<Item = I> {
    let mut count = 0;
    let max = n - 1;

    s.filter(move |_| {
        let max_reached = count == max;

        count = if max_reached { 0 } else { count + 1 };

        future::ready(max_reached)
    })
}
