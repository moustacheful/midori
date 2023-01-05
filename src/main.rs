use futures::{future::select_all, stream, Stream, StreamExt};
use std::{
    pin::Pin,
    time::{Duration, SystemTime},
};
use tokio::time::Interval;
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

    pub fn subdiv(&mut self, factor: u64) -> impl Stream<Item = u64> {
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
            Box::pin(tempo.subdiv(1).map(|instant| Wrapper::Value(1))), // normal tempo
            Box::pin(tempo.subdiv(4).map(|instant| Wrapper::Value(4))), // tempo / 4
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

    pub async fn listen(
        self,
        origin_stream: Pin<Box<dyn Stream<Item = Wrapper<u64>> + Send>>,
    ) -> Option<()> {
        let name = self.name.clone();
        let mut tempo = Tempo::new(60);

        println!("{:?} listening", &name);

        let mut r = self
            .transforms
            .iter()
            .fold(origin_stream, |acc, transform| {
                transform.pipe(acc, &mut tempo)
            })
            .map(|v| {
                println!("{:?} - {:?}", &v, &name,);
                v
            });

        // Loop through events -- this should happen outside and leave this as a stream only.
        while let Some(item) = r.next().await {}

        println!("FINISHED {:?}", &name);

        Some(())
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
        let rx = self.rx.clone();

        let futs = pipelines
            .into_iter()
            .map(|p| {
                let origin_stream: Pin<Box<dyn Stream<Item = Wrapper<u64>> + Send>> =
                    Box::pin(rx.clone().into_stream().map(|v| Wrapper::Value(v)));

                tokio::spawn(async { Box::pin(p.listen(origin_stream)).await })
            })
            .collect::<Vec<_>>();

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
                // Pipeline::new("Two".to_string()),
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
