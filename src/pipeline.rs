use futures::StreamExt;
use futures::{future, Stream};
use std::pin::Pin;

use crate::{tempo, transforms::Transform};

use super::Wrapper;

pub(crate) struct Pipeline {
    pub(crate) rx: flume::Receiver<Wrapper<u64>>,
    pub tx: flume::Sender<Wrapper<u64>>,
    pub(crate) name: String,
    pub transforms: Vec<Box<dyn Transform + Sync + Send>>,
}

impl Pipeline {
    pub(crate) fn pipe_stream(
        origin_stream: Pin<Box<dyn Stream<Item = Wrapper<u64>> + Send>>,
        tempo: &mut tempo::Tempo,
        mut transform: Box<dyn Transform + Sync + Send>,
    ) -> Pin<Box<dyn Stream<Item = Wrapper<u64>> + Send>> {
        let streams: Vec<Pin<Box<dyn Stream<Item = Wrapper<u64>> + Send>>> = vec![
            origin_stream,
            Box::pin(tempo.subdiv(32).map(|_| Wrapper::Tempo)),
        ];

        let stream = futures::stream::select_all::select_all(streams)
            .filter_map(move |v| future::ready(transform.process_message(v)));

        Box::pin(stream)
    }

    pub fn new(name: String, transforms: Vec<Box<dyn Transform + Sync + Send>>) -> Pipeline {
        let (tx, rx) = flume::unbounded::<Wrapper<u64>>();
        Pipeline {
            tx,
            rx,
            name,
            transforms,
        }
    }

    pub async fn listen(self) -> impl Stream<Item = Wrapper<u64>> {
        let name = self.name.clone();
        let mut tempo = tempo::Tempo::new(60);
        let origin_stream: Pin<Box<dyn Stream<Item = Wrapper<u64>> + Send>> =
            Box::pin(self.rx.into_stream());
        println!("{:?} listening", &name);

        self.transforms
            .into_iter()
            .fold(origin_stream, move |acc, transform| {
                Self::pipe_stream(acc, &mut tempo, transform)
            })
    }
}
