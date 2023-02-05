use std::collections::HashMap;

use crate::{
    midi_mapper::MidiRouterMessage,
    pipeline::{Pipeline, PipelineOptions},
    tempo::Clock,
};
use futures::{future::select_all, StreamExt};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    pub input_devices: HashMap<String, String>,
    pub output_devices: HashMap<String, String>,
    pub pipelines: Vec<PipelineOptions>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum MidiRouterMessageWrapper {
    Tick,
    RouterMessage(MidiRouterMessage),
}

pub struct App {
    pub egress: Option<flume::Sender<MidiRouterMessage>>,
    pub ingress: Option<flume::Receiver<MidiRouterMessage>>,
    pub pipelines: Vec<Pipeline>,
}

impl App {
    pub fn from_config(config: AppConfig) -> App {
        App {
            ingress: None,
            egress: None,
            pipelines: config
                .pipelines
                .into_iter()
                .map(|pipeline_config| Pipeline::from_config(pipeline_config))
                .collect(),
        }
    }

    pub fn set_ingress(&mut self, ingress: flume::Receiver<MidiRouterMessage>) {
        self.ingress = Some(ingress);
    }

    pub fn set_egress(&mut self, egress: flume::Sender<MidiRouterMessage>) {
        self.egress = Some(egress);
    }

    pub async fn run(self) -> Option<()> {
        let (clock, clock_handler) = Clock::new(60.0, 96.0);
        let ingress = self.ingress.unwrap();
        let egress = self.egress.unwrap();

        tokio::spawn(async move { clock.start().await });

        // Collect each pipelines' sender
        let txs: Vec<flume::Sender<MidiRouterMessageWrapper>> =
            self.pipelines.iter().map(|p| p.tx.clone()).collect::<_>();

        // Broadcast events from ingress to each pipeline sender
        tokio::spawn(async move {
            while let Ok(x) = ingress.recv_async().await {
                txs.iter().for_each(|tx| {
                    tx.send(MidiRouterMessageWrapper::RouterMessage(x.clone()))
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
                        if let MidiRouterMessageWrapper::RouterMessage(message) = x {
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
