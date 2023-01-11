use futures::{future, future::select_all, Stream, StreamExt};
use std::{pin::Pin, time::Duration};
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
        Box::pin(tempo.subdiv(32).map(|_| Wrapper::Tempo)),
    ];

    let stream = futures::stream::select_all::select_all(streams)
        .filter_map(move |v| future::ready(transform.process_message(v)));

    Box::pin(stream)
}

pub trait Transform {
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

struct InspectTransform {
    prefix: String,
}

impl Transform for InspectTransform {
    fn on_message(&mut self, v: Wrapper<u64>) -> Option<Wrapper<u64>> {
        println!("[{:?}]: {:?}", self.prefix, v);

        Some(v)
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
        let current_index = self
            .pressed_keys
            .get(self.current_index % self.pressed_keys.len());

        self.current_index += 1;
        Some(Wrapper::Value(current_index.unwrap().clone()))
    }

    fn on_message(&mut self, v: Wrapper<u64>) -> Option<Wrapper<u64>> {
        if let Wrapper::Value(key) = v {
            self.pressed_keys.push(key);
            self.current_index = 0;
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
    rx: flume::Receiver<Wrapper<u64>>,
    pub tx: flume::Sender<Wrapper<u64>>,
    name: String,
    pub transforms: Vec<Box<dyn Transform + Sync + Send>>,
}
impl Pipeline {
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
        let mut tempo = Tempo::new(60);
        let origin_stream: Pin<Box<dyn Stream<Item = Wrapper<u64>> + Send>> =
            Box::pin(self.rx.into_stream());
        println!("{:?} listening", &name);

        self.transforms
            .into_iter()
            .fold(origin_stream, move |acc, transform| {
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
        let txs: Vec<flume::Sender<Wrapper<u64>>> =
            self.pipelines.iter().map(|p| p.tx.clone()).collect::<_>();

        tokio::spawn(async move {
            while let Ok(x) = self.ingress_rx.recv_async().await {
                txs.iter().for_each(|tx| {
                    tx.send(Wrapper::Value(x)).unwrap();
                });
            }
        });

        let pipeline_futures = self
            .pipelines
            .into_iter()
            .map(|p| {
                let egress = self.egress_sender.clone();

                tokio::spawn(async move {
                    let mut result_stream = p.listen().await;

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
        let app = App::new(
            egress_sender,
            ingress_receiver,
            vec![
                Pipeline::new(
                    "One".to_string(),
                    vec![
                        Box::new(ArpeggioTransform::new()),
                        Box::new(InspectTransform {
                            prefix: "P1".to_string(),
                        }),
                    ],
                ),
                Pipeline::new(
                    "Two".to_string(),
                    vec![Box::new(InspectTransform {
                        prefix: "P2".to_string(),
                    })],
                ),
            ],
        );

        app.run().await;

        Some(())
    });

    tokio::spawn(async move {
        let mut i = tokio::time::interval(Duration::from_millis(5000));
        let sender = ingress_sender.clone();
        let mut n = 0;

        while let _ = i.tick().await {
            sender.send(n);
            n += 1;
        }
    });

    // This receiver will receive messages from all pipelines, it's up to whoever consumes this
    // To dispatch the messages over to its destination.
    while let Ok(d) = egress_receiver.recv_async().await {
        // println!("{:?}", d);
    }

    f.await.unwrap();
}
