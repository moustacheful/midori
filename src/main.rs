use futures::{future::select_all, Stream, StreamExt};
use std::{pin::Pin, time::Duration};
use tokio_stream::wrappers::IntervalStream;

struct Tempo {
    main: IntervalStream,
    beat_interval: u64,
}

impl Tempo {
    fn get_interval_stream(interval_ms: u64) -> IntervalStream {
        println!("beginning tempo with {}", interval_ms);
        let interval = tokio::time::interval(Duration::from_millis(interval_ms));

        tokio_stream::wrappers::IntervalStream::new(interval)
    }

    pub fn new(bpm: u64) -> Tempo {
        let beat_interval = ((bpm as f64 / 70.0) * 1000.0) as u64;
        let main = Self::get_interval_stream(beat_interval);

        Tempo {
            main,
            beat_interval,
        }
    }

    pub async fn subdiv(&mut self, factor: u64) -> IntervalStream {
        self.main.next().await;

        Self::get_interval_stream(self.beat_interval / factor)
    }
}

#[derive(Debug)]
struct Transform {}

impl Transform {
    pub fn on_tempo() {}

    pub fn on_message() {}
}

impl Stream for Transform {
    type Item = u64;

    fn poll_next(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        std::task::Poll::Pending
    }
}

#[derive(Debug)]
struct Pipeline {
    name: String,
    pub tx: flume::Sender<u64>,
    pub rx: flume::Receiver<u64>,
    pub transforms: Vec<Transform>,
}

#[derive(Debug)]
enum Wrapper<T> {
    Tempo,
    Value(T),
}
impl Pipeline {
    pub fn new(name: String) -> Pipeline {
        let (tx, rx) = flume::unbounded::<u64>();

        Pipeline {
            tx,
            rx,
            name,
            transforms: vec![Transform {}, Transform {}],
        }
    }

    pub async fn listen(self) {
        let name = self.name.clone();
        let mut tempo = Tempo::new(60);

        let streams: Vec<Pin<Box<dyn Stream<Item = Wrapper<u64>> + Send>>> = vec![
            Box::pin(self.rx.into_stream().map(|val| Wrapper::Value(val))),
            Box::pin(tempo.subdiv(1).await.map(|instant| Wrapper::Value(1))),
            Box::pin(tempo.subdiv(2).await.map(|instant| Wrapper::Value(2))),
            Box::pin(tempo.subdiv(8).await.map(|instant| Wrapper::Value(8))),
        ];

        let mut stream = futures::stream::select_all::select_all(streams);

        let transforms = self.transforms;

        println!("{:?} listening", &name);

        while let Some(item) = stream.next().await {
            transforms.iter().for_each(|t| {
                // dbg!(t);
            });

            println!("{:?} - by {:?}", item, &name);
        }
    }
}

struct App {
    pub tx: flume::Sender<u64>,
    pub rx: flume::Receiver<u64>,
    pub pipelines: Vec<Pipeline>,
}

impl App {
    pub fn new(tx: flume::Sender<u64>, rx: flume::Receiver<u64>, pipelines: Vec<Pipeline>) -> App {
        App { tx, rx, pipelines }
    }

    pub async fn run_broadcast(self) -> Option<()> {
        let pipelines = self.pipelines;
        let txs = pipelines.iter().map(|p| p.tx.clone()).collect::<Vec<_>>();
        let rx = self.rx.clone();

        let futs = pipelines
            .into_iter()
            .map(|p| tokio::spawn(async { Box::pin(p.listen()).await }))
            .collect::<Vec<_>>();

        while let Ok(item) = rx.recv_async().await {
            for tx in &txs {
                tx.send(item.clone()).unwrap();
            }
        }

        select_all(futs).await;

        Some(())
    }
}

#[tokio::main(flavor = "multi_thread", worker_threads = 4)]
async fn main() {
    let (tx, rx) = flume::unbounded::<u64>();

    let app_tx = tx.clone();
    let app_rx = rx.clone();

    let f = tokio::spawn(async {
        let app = App::new(
            app_tx,
            app_rx,
            vec![
                Pipeline::new("One".to_string()),
                Pipeline::new("Two".to_string()),
            ],
        );

        app.run_broadcast().await;

        Some(())
    });

    println!("Sending messages");
    tx.send(10).unwrap();
    tx.send(20).unwrap();
    tx.send(30).unwrap();
    tx.send(40).unwrap();
    tx.send(50).unwrap();
    tx.send(60).unwrap();

    f.await.unwrap();
}
