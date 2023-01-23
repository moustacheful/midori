use futures::StreamExt;
use futures::{future, Stream};
use std::pin::Pin;

use crate::MidiRouterMessageWrapper;
use crate::{tempo, transforms::Transform};

pub struct Pipeline {
    pub(crate) rx: flume::Receiver<MidiRouterMessageWrapper>,
    pub tx: flume::Sender<MidiRouterMessageWrapper>,
    pub(crate) name: String,
    pub transforms: Vec<Box<dyn Transform + Sync + Send>>,
}

impl Pipeline {
    pub(crate) fn pipe_stream(
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

    pub fn new(name: String, transforms: Vec<Box<dyn Transform + Sync + Send>>) -> Pipeline {
        let (tx, rx) = flume::unbounded::<MidiRouterMessageWrapper>();
        Pipeline {
            tx,
            rx,
            name,
            transforms,
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
