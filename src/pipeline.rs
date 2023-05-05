use futures::StreamExt;
use futures::{future, Stream};
use schemars::JsonSchema;
use serde::Deserialize;
use std::pin::Pin;

use crate::app::MIDIMapperEvent;
use crate::scheduler::Scheduler;
use crate::tempo::ClockHandler;
use crate::transforms::transform::SerializedTransform;
use crate::transforms::Transform;
use crate::transforms::{
    ArpeggioTransform, DistributeTransform, FilterTransform, InspectTransform, MapTransform,
    OutputTransform, WasmTransform,
};

#[derive(Debug, Deserialize, JsonSchema)]
pub struct PipelineOptions {
    pub name: String,
    pub transforms: Vec<SerializedTransform>,
}

pub struct Pipeline {
    pub rx: flume::Receiver<MIDIMapperEvent>,
    pub tx: flume::Sender<MIDIMapperEvent>,
    pub name: String,
    pub transforms: Vec<Box<dyn Transform + Send>>,
}

impl Pipeline {
    pub fn pipe_stream(
        origin_stream: Pin<Box<dyn Stream<Item = MIDIMapperEvent> + Send>>,
        clock: &ClockHandler,
        mut transform: Box<dyn Transform + Send>,
    ) -> Pin<Box<dyn Stream<Item = MIDIMapperEvent> + Send>> {
        let mut streams: Vec<Pin<Box<dyn Stream<Item = MIDIMapperEvent> + Send>>> =
            vec![origin_stream];

        if let Some(subdiv) = transform.get_tempo_subdiv() {
            streams.push(Box::pin(
                clock.create(subdiv).map(|_| MIDIMapperEvent::Tick),
            ))
        }

        let (scheduler, scheduler_handler) = Scheduler::new();

        transform.set_scheduler(scheduler_handler.clone());

        let stream = futures::stream::select_all::select_all(streams).filter_map(move |v| {
            let result = transform
                .process_message(v, &scheduler_handler)
                .map(MIDIMapperEvent::RouterMessage);

            future::ready(result)
        });

        let output_streams: Vec<Pin<Box<dyn Stream<Item = MIDIMapperEvent> + Send>>> =
            vec![Box::pin(stream), Box::pin(scheduler.stream())];

        Box::pin(futures::stream::select_all(output_streams))
    }

    pub fn from_config(config: PipelineOptions) -> Self {
        let (tx, rx) = flume::unbounded::<MIDIMapperEvent>();

        Self {
            tx,
            rx,
            name: config.name,
            transforms: config
                .transforms
                .into_iter()
                .map(|transform_config| {
                    let transform: Box<dyn Transform + Send> = match transform_config {
                        SerializedTransform::Filter(config) => {
                            Box::new(FilterTransform::from_config(config))
                        }

                        SerializedTransform::Arpeggio(config) => {
                            Box::new(ArpeggioTransform::from_config(config))
                        }

                        SerializedTransform::Map(config) => {
                            Box::new(MapTransform::from_config(config))
                        }

                        SerializedTransform::Distribute(config) => {
                            Box::new(DistributeTransform::from_config(config))
                        }

                        SerializedTransform::Output(config) => {
                            Box::new(OutputTransform::from_config(config))
                        }

                        SerializedTransform::Inspect(config) => {
                            Box::new(InspectTransform::from_config(config))
                        }

                        SerializedTransform::Wasm(config) => {
                            Box::new(WasmTransform::from_config(config))
                        }
                    };

                    transform
                })
                .collect(),
        }
    }

    pub async fn listen(self, clock: ClockHandler) -> impl Stream<Item = MIDIMapperEvent> {
        let name = self.name.clone();
        let origin_stream: Pin<Box<dyn Stream<Item = MIDIMapperEvent> + Send>> =
            Box::pin(self.rx.into_stream());
        println!("{:?} listening", &name);

        self.transforms
            .into_iter()
            .fold(origin_stream, move |acc, transform| {
                Self::pipe_stream(acc, &clock, transform)
            })
    }
}
