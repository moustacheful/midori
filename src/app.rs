use std::collections::HashMap;

use crate::{
    midi_event::{MIDIEvent, MIDIRouterEvent},
    pipeline::{Pipeline, PipelineOptions},
    tempo::{ClockEvent, ClockHandler, ExternalClock, InternalClock},
};
use futures::{future::select_all, StreamExt};
use schemars::JsonSchema;
use serde::Deserialize;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ClockConfig {
    bpm: Option<u64>,
    ppqn: Option<u64>,
    from: Option<String>,
    to: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct AppConfig {
    pub input_devices: HashMap<String, String>,
    pub output_devices: HashMap<String, String>,
    pub pipelines: Vec<PipelineOptions>,
    pub clock: Option<ClockConfig>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum MIDIMapperEvent {
    Tick,
    RouterMessage(MIDIRouterEvent),
}

pub struct App {
    pub egress: Option<flume::Sender<MIDIRouterEvent>>,
    pub ingress: Option<flume::Receiver<MIDIRouterEvent>>,
    pub pipelines: Vec<Pipeline>,
    clock_source: String,
    clock_destination: Vec<String>,
    clock_handler: ClockHandler,
    internal_clock: Option<InternalClock>,
    external_clock: Option<ExternalClock>,
}

impl App {
    pub fn process_clock_config(
        clock: &ClockConfig,
    ) -> (Option<InternalClock>, Option<ExternalClock>, ClockHandler) {
        if clock.bpm.is_some() && clock.from.is_some() {
            panic!("Clock cannot have set bpm and set to external at the same time.");
        }

        let mut internal_clock: Option<InternalClock> = None;
        let mut external_clock: Option<ExternalClock> = None;

        let clock_handler = if let (Some(bpm), Some(ppqn)) = (clock.bpm, clock.ppqn) {
            let (clock, clock_handler) = InternalClock::new(bpm as f64, ppqn as f64);

            internal_clock = Some(clock);
            clock_handler
        } else if let Some(ppqn) = clock.ppqn {
            let (clock, clock_handler) = ExternalClock::new(ppqn as f64);

            external_clock = Some(clock);
            clock_handler
        } else {
            panic!("Either bpm/ppqn or from/ppqn must be set");
        };

        (internal_clock, external_clock, clock_handler)
    }

    pub fn from_config(config: AppConfig) -> App {
        let clock_config = config.clock.unwrap_or(ClockConfig {
            bpm: Some(60),
            ppqn: Some(48),
            from: None,
            to: None,
        });

        let (internal_clock, external_clock, clock_handler) =
            Self::process_clock_config(&clock_config);

        App {
            ingress: None,
            egress: None,
            clock_source: clock_config.from.unwrap_or("self".into()),
            clock_destination: clock_config.to.unwrap_or(vec![]),
            clock_handler,
            internal_clock,
            external_clock,
            pipelines: config
                .pipelines
                .into_iter()
                .map(Pipeline::from_config)
                .collect(),
        }
    }

    pub fn set_ingress(&mut self, ingress: flume::Receiver<MIDIRouterEvent>) {
        self.ingress = Some(ingress);
    }

    pub fn set_egress(&mut self, egress: flume::Sender<MIDIRouterEvent>) {
        self.egress = Some(egress);
    }

    pub async fn run(self) -> Option<()> {
        let ingress = self.ingress.unwrap();
        let egress = self.egress.unwrap();
        let clock_egress = egress.clone();
        let clock_handler = self.clock_handler;
        let clock_sender = self.external_clock.and_then(|v| Some(v.sender.clone()));
        if let Some(internal_clock) = self.internal_clock {
            tokio::spawn(async move { internal_clock.start().await });
        }

        // Collect each pipelines' sender
        let txs: Vec<flume::Sender<MIDIMapperEvent>> =
            self.pipelines.iter().map(|p| p.tx.clone()).collect::<_>();

        // Broadcast events from ingress to each pipeline sender
        tokio::spawn(async move {
            while let Ok(router_event) = ingress.recv_async().await {
                // If this device matches with our clock source, and is a sysex event
                // event send it directly to the destination devices
                //
                // This will skip pipelines altogether for clock events
                // which both should ensure minimum latency is introduced to clocks
                // and reduce the amount of work the pipelines need to do.
                if router_event.event.is_sysex() {
                    if router_event.device != self.clock_source {
                        continue;
                    }

                    let sender = clock_sender.as_ref().unwrap();
                    if matches!(router_event.event, MIDIEvent::TimingClock) {
                        sender.send(ClockEvent::Tick).ok();
                    }

                    if matches!(router_event.event, MIDIEvent::PlaybackPosition(_)) {
                        println!("Received position {:?}", router_event);
                        sender.send(ClockEvent::Restart).ok();
                    }

                    // self.clock_destination
                    //     .iter()
                    //     .for_each(|destination_device| {
                    //         let mut new_router_event = router_event.clone();
                    //         new_router_event.device = destination_device.into();
                    //         clock_egress.send(new_router_event).unwrap();
                    //     });
                    continue;
                }

                txs.iter().for_each(|tx| {
                    tx.send(MIDIMapperEvent::RouterMessage(router_event.clone()))
                        .unwrap();
                });
            }
        });

        // Iterate through all pipelines and obtain their streams
        // Listen to all their messages and send them to the egress
        let pipeline_futures = self
            .pipelines
            .into_iter()
            .map(|p| {
                let egress = egress.clone();
                let local_clock = clock_handler.clone();
                tokio::spawn(async move {
                    let mut result_stream = p.listen(local_clock).await;

                    while let Some(x) = result_stream.next().await {
                        if let MIDIMapperEvent::RouterMessage(message) = x {
                            egress.send(message).unwrap();
                        }
                    }
                })
            })
            .collect::<Vec<_>>();

        // Should this be the return instead?
        let _ = select_all(pipeline_futures).await;

        Some(())
    }
}
