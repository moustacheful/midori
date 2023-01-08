use futures::{future::select_all, Stream, StreamExt};
use std::{ops::Deref, pin::Pin, time::Duration};
use tokio_stream::wrappers::IntervalStream;

/**
 * This is some kind of internal clock?
 * e.g. when we have no device as source of it all.
 */
pub struct Tempo {
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

pub fn pipe_stream(
    origin_stream: Pin<Box<dyn Stream<Item = Wrapper<u64>> + Send>>,
    tempo: &mut Tempo,
    mut transform: Box<dyn Transform + Sync + Send>,
) -> Pin<Box<dyn Stream<Item = Wrapper<u64>> + Send>> {
    let streams: Vec<Pin<Box<dyn Stream<Item = Wrapper<u64>> + Send>>> = vec![
        origin_stream,
        Box::pin(tempo.subdiv(1).map(|_| Wrapper::Value(2))),
    ];

    let stream = futures::stream::select_all::select_all(streams)
        .filter_map(|v| async move { transform.process_message(v) });

    Box::pin(stream)
}

trait Transform {
    // This triggers on what we subscribe as points of interest, e.g. an arpeggio?
    fn on_tick(&mut self, v: Wrapper<u64>) -> Option<Wrapper<u64>> {
        None
    }

    fn on_message(&mut self, v: Wrapper<u64>) -> Option<Wrapper<u64>> {
        Some(v)
    }

    fn process_message(&mut self, message: Wrapper<u64>) -> Option<Wrapper<u64>> {
        match message {
            Wrapper::Tempo => self.on_tick(message),
            Wrapper::Value(_) => self.on_message(message),
        }
    }
}

#[derive(Debug)]
struct ArpeggioTransform {
    pressed_keys: Vec<u64>,
    current_index: usize,
}

impl ArpeggioTransform {
    pub fn new() -> ArpeggioTransform {
        ArpeggioTransform {
            pressed_keys: vec![],
            current_index: 0,
        }
    }
}

impl Transform for ArpeggioTransform {
    fn on_tick(&mut self, v: Wrapper<u64>) -> Option<Wrapper<u64>> {
        let current_index = self.pressed_keys.get(self.current_index);

        if None == current_index {
            return None;
        }

        self.current_index += 1;
        Some(Wrapper::Value(current_index.unwrap().clone()))
    }

    fn on_message(&mut self, v: Wrapper<u64>) -> Option<Wrapper<u64>> {
        // Match only key on/off...
        match v {
            Wrapper::Tempo => todo!(),
            Wrapper::Value(value) => self.pressed_keys.push(value),
        }

        None
    }
}

#[derive(Debug)]
pub enum Wrapper<T> {
    Tempo,
    Value(T),
}

struct Pipeline {
    name: String,
    pub transforms: Vec<Box<dyn Transform + Sync + Send + 'static>>,
}
impl Pipeline {
    pub fn new(name: String) -> Pipeline {
        Pipeline {
            name,
            transforms: vec![Box::new(ArpeggioTransform::new())],
        }
    }

    pub async fn listen(
        self,
        origin_stream: Pin<Box<dyn Stream<Item = Wrapper<u64>> + Send>>,
    ) -> impl Stream<Item = Wrapper<u64>> {
        let name = self.name.clone();
        let mut tempo = Tempo::new(60);

        println!("{:?} listening", &name);

        let mut transforms = self.transforms;

        transforms
            .into_iter()
            .fold(origin_stream, move |acc, mut transform| {
                pipe_stream(acc, &mut tempo, transform)
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
