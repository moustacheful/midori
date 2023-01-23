mod midi_event;
mod midi_mapper;
mod pipeline;
mod tempo;
mod transforms;

use futures::{future::select_all, StreamExt};
use midi_mapper::{MidiMapper, MidiRouterMessage};
use pipeline::Pipeline;
use transforms::{
    ArpeggioTransform, DistributeTransform, FilterTransform, FilterTransformOptions, MapTransform,
    MapTransformOptions, OutputTransform,
};

#[derive(Debug)]
pub enum Wrapper<T> {
    Tempo,
    Value(T),
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum MidiRouterMessageWrapper {
    Tick,
    RouterMessage(MidiRouterMessage),
}

pub struct App {
    pub egress_sender: Option<flume::Sender<MidiRouterMessage>>,
    pub ingress_rx: Option<flume::Receiver<MidiRouterMessage>>,
    pub pipelines: Vec<pipeline::Pipeline>,
}

impl App {
    pub fn new(pipelines: Vec<pipeline::Pipeline>) -> App {
        App {
            ingress_rx: None,
            egress_sender: None,
            pipelines,
        }
    }

    pub fn set_ingress(&mut self, ingress_rx: flume::Receiver<MidiRouterMessage>) {
        self.ingress_rx = Some(ingress_rx);
    }

    pub fn set_egress(&mut self, egress_sender: flume::Sender<MidiRouterMessage>) {
        self.egress_sender = Some(egress_sender);
    }

    pub async fn run(self) -> Option<()> {
        let ingress_rx = self.ingress_rx.unwrap();
        let egress_sender = self.egress_sender.unwrap();

        // Collect each pipelines' sender
        let txs: Vec<flume::Sender<MidiRouterMessageWrapper>> =
            self.pipelines.iter().map(|p| p.tx.clone()).collect::<_>();

        // Broadcast events from ingress to each pipeline sender
        tokio::spawn(async move {
            while let Ok(x) = ingress_rx.recv_async().await {
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
                let egress = egress_sender.clone();

                tokio::spawn(async move {
                    let mut result_stream = p.listen().await;

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

// How many threads should I use here...?
#[tokio::main(flavor = "multi_thread", worker_threads = 4)]
async fn main() {
    let pipelines = vec![
        Pipeline::new(
            "One".to_string(),
            vec![
                Box::new(FilterTransform::new(FilterTransformOptions {
                    channels: vec![0],
                })),
                Box::new(ArpeggioTransform::new(Some(8))),
                Box::new(DistributeTransform::new(vec![0, 1, 2])),
                Box::new(OutputTransform::new("modelcycles-out".to_string())),
            ],
        ),
        Pipeline::new(
            "Two".to_string(),
            vec![
                Box::new(FilterTransform::new(FilterTransformOptions {
                    channels: vec![9],
                })),
                Box::new(MapTransform::new(MapTransformOptions {
                    channels: vec![(9, 5)],
                    cc: vec![],
                })),
                Box::new(OutputTransform::new("modelcycles-out".to_string())),
            ],
        ),
        Pipeline::new(
            "Three".to_string(),
            vec![
                Box::new(FilterTransform::new(FilterTransformOptions {
                    channels: vec![8],
                })),
                Box::new(MapTransform::new(MapTransformOptions {
                    cc: vec![(74, 16)],
                    channels: vec![(8, 4)],
                })),
                Box::new(ArpeggioTransform::new(Some(2))),
                Box::new(OutputTransform::new("modelcycles-out".to_string())),
            ],
        ),
    ];

    let app = App::new(pipelines);

    let mut midi_mapper = MidiMapper::new();
    midi_mapper.add_output("Artiphon Orba".to_string(), "orba-out".to_string());
    midi_mapper.add_input("Artiphon Orba".to_string(), "orba".to_string());
    midi_mapper.add_output("Elektron".to_string(), "modelcycles-out".to_string());

    midi_mapper.start(app);
}
