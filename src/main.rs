use futures::{future::select_all, Stream, StreamExt};
use std::{pin::Pin, time::Duration};
use tokio_stream::wrappers::IntervalStream;

/**
 * This is some kind of internal clock?
 * e.g. when we have no device as source of it all.
 */
struct Tempo {
    main: IntervalStream,
    beat_interval: u64,
}

impl Tempo {
    fn get_interval_stream(interval_ms: u64) -> IntervalStream {
        let interval = tokio::time::interval(Duration::from_millis(interval_ms));

        tokio_stream::wrappers::IntervalStream::new(interval)
    }

    pub fn new(bpm: u64) -> Tempo {
        let beat_interval = ((bpm as f64 / 60.0) * 1000.0) as u64;
        let main = Self::get_interval_stream(beat_interval);

        Tempo {
            main,
            beat_interval,
        }
    }

    pub fn subdiv(&mut self, factor: u64) -> impl Stream<Item = u64> {
        // This seems wrong... but we want to remove the need for owning self at the end
        futures::executor::block_on(self.main.next());

        Self::get_interval_stream(self.beat_interval.clone() / factor).map(|_| 10 as u64)
    }
}

#[derive(Debug)]
struct Transform {}

impl Transform {
    fn on_tempo(v: Wrapper<u64>) -> Option<Wrapper<u64>> {
        None
    }

    fn on_message(v: Wrapper<u64>) -> Option<Wrapper<u64>> {
        Some(v)
    }

    pub fn pipe(
        &self,
        origin_stream: Pin<Box<dyn Stream<Item = Wrapper<u64>> + Send>>,
        tempo: &mut Tempo,
    ) -> Pin<Box<dyn Stream<Item = Wrapper<u64>> + Send>> {
        let streams: Vec<Pin<Box<dyn Stream<Item = Wrapper<u64>> + Send>>> = vec![
            origin_stream,
            Box::pin(tempo.subdiv(1).map(|_| Wrapper::Value(2))),
        ];

        let stream = futures::stream::select_all::select_all(streams).filter_map(|v| async move {
            match v {
                Wrapper::Tempo => Self::on_tempo(v),
                Wrapper::Value(_) => Self::on_message(v),
            }
        });

        Box::pin(stream)
    }
}

#[derive(Debug)]
enum Wrapper<T> {
    Tempo,
    Value(T),
}

struct Pipeline {
    name: String,
    pub transforms: Vec<Transform>,
}
impl Pipeline {
    pub fn new(name: String) -> Pipeline {
        Pipeline {
            name,
            transforms: vec![Transform {}],
        }
    }

    pub fn add_transforms(&mut self, transforms: Vec<Transform>) {
        self.transforms = transforms;
    }

    pub async fn listen(
        self,
        origin_stream: Pin<Box<dyn Stream<Item = Wrapper<u64>> + Send>>,
    ) -> impl Stream<Item = Wrapper<u64>> {
        let name = self.name.clone();
        let mut tempo = Tempo::new(60);

        println!("{:?} listening", &name);

        self.transforms
            .iter()
            .fold(origin_stream, |acc, transform| {
                transform.pipe(acc, &mut tempo)
            })
    }
}

struct App {
    pub egress_sender: flume::Sender<Wrapper<u64>>,
    pub ingress_rx: flume::Receiver<u64>,
    pub pipelines: Vec<Pipeline>,
}

impl App {
    pub fn new(
        egress_sender: flume::Sender<Wrapper<u64>>,
        ingress_rx: flume::Receiver<u64>,
        pipelines: Vec<Pipeline>,
    ) -> App {
        App {
            ingress_rx,
            egress_sender,
            pipelines,
        }
    }

    pub async fn run(self) -> Option<()> {
        let pipeline_futures = self
            .pipelines
            .into_iter()
            .map(|p| {
                let egress = self.egress_sender.clone();
                let origin_stream: Pin<Box<dyn Stream<Item = Wrapper<u64>> + Send>> = Box::pin(
                    self.ingress_rx
                        .clone()
                        .into_stream()
                        .map(|v| Wrapper::Value(v)),
                );

                tokio::spawn(async move {
                    let mut result_stream = p.listen(origin_stream).await;

                    while let Some(x) = result_stream.next().await {
                        egress.send(x).unwrap();
                    }
                })
            })
            .collect::<Vec<_>>();

        // Should this be the return instead?
        select_all(pipeline_futures).await;

        Some(())
    }
}

#[tokio::main(flavor = "multi_thread", worker_threads = 4)]
async fn main() {
    // ingress_sender should be used to send midi events into the pipeline
    let (ingress_sender, ingress_receiver) = flume::unbounded::<u64>();
    let (egress_sender, egress_receiver) = flume::unbounded::<Wrapper<u64>>();

    let f = tokio::spawn(async {
        // Pipelines should be created by parsing a json
        let mut pipeline = Pipeline::new("One".to_string());
        pipeline.add_transforms(vec![Transform {}]);

        let app = App::new(egress_sender, ingress_receiver, vec![pipeline]);

        app.run().await;

        Some(())
    });

    // This receiver will receive messages from all pipelines, it's up to whoever consumes this
    // To dispatch the messages over to its destination.
    while let Ok(d) = egress_receiver.recv_async().await {
        println!("{:?}", d);
    }

    f.await.unwrap();
}
