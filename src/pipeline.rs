use futures::StreamExt;
use futures::{future, Stream};
use serde::Deserialize;
use std::pin::Pin;

use crate::app::MidiRouterMessageWrapper;
use crate::transforms::transform::SerializedTransform;
use crate::transforms::{ArpeggioTransform, FilterTransform, MapTransform, OutputTransform};
use crate::{tempo, transforms::Transform};

#[derive(Debug, Deserialize)]
pub struct PipelineOptions {
    pub name: String,
    pub transforms: Vec<SerializedTransform>,
}

pub struct Pipeline {
    pub rx: flume::Receiver<MidiRouterMessageWrapper>,
    pub tx: flume::Sender<MidiRouterMessageWrapper>,
    pub name: String,
    pub transforms: Vec<Box<dyn Transform + Sync + Send>>,
}

impl Pipeline {
    pub fn pipe_stream(
        origin_stream: Pin<Box<dyn Stream<Item = MidiRouterMessageWrapper> + Send>>,
        tempo: &mut tempo::Tempo,
        mut transform: Box<dyn Transform + Sync + Send>,
    ) -> Pin<Box<dyn Stream<Item = MidiRouterMessageWrapper> + Send>> {
        let mut streams: Vec<Pin<Box<dyn Stream<Item = MidiRouterMessageWrapper> + Send>>> =
            vec![origin_stream];

        if let Some(subdiv) = transform.get_tempo_subdiv() {
            streams.push(Box::pin(
                tempo.subdiv(subdiv).map(|_| MidiRouterMessageWrapper::Tick),
            ))
        }

        let stream = futures::stream::select_all::select_all(streams).filter_map(move |v| {
            let result = match transform.process_message(v) {
                Some(r) => Some(MidiRouterMessageWrapper::RouterMessage(r)),
                None => None,
            };

            future::ready(result)
        });

        Box::pin(stream)
    }

    pub fn from_config(config: PipelineOptions) -> Self {
        let (tx, rx) = flume::unbounded::<MidiRouterMessageWrapper>();

        Self {
            tx,
            rx,
            name: config.name,
            transforms: config
                .transforms
                .into_iter()
                .map(|transform_config| {
                    let transform: Box<dyn Transform + Sync + Send> = match transform_config {
                        SerializedTransform::Filter(config) => {
                            Box::new(FilterTransform::from_config(config))
                        }

                        SerializedTransform::Arpeggio(config) => {
                            Box::new(ArpeggioTransform::from_config(config))
                        }

                        SerializedTransform::Map(config) => {
                            Box::new(MapTransform::from_config(config))
                        }

                        SerializedTransform::Output(config) => {
                            Box::new(OutputTransform::from_config(config))
                        }
                    };

                    transform
                })
                .collect(),
        }
    }

    pub async fn listen(self) -> impl Stream<Item = MidiRouterMessageWrapper> {
        let name = self.name.clone();
        let mut tempo = tempo::Tempo::new(60);
        let origin_stream: Pin<Box<dyn Stream<Item = MidiRouterMessageWrapper> + Send>> =
            Box::pin(self.rx.into_stream());
        println!("{:?} listening", &name);

        self.transforms
            .into_iter()
            .fold(origin_stream, move |acc, transform| {
                Self::pipe_stream(acc, &mut tempo, transform)
            })
    }
}
