use futures::{future::select_all, StreamExt};
use std::time::Duration;

#[derive(Debug)]
struct Pipeline {
    name: String,
    pub tx: flume::Sender<u64>,
    pub rx: flume::Receiver<u64>,
}

impl Pipeline {
    pub fn new(name: String) -> Pipeline {
        let (tx, rx) = flume::unbounded::<u64>();

        Pipeline { tx, rx, name }
    }

    pub async fn listen(self) {
        let name = self.name.clone();
        let mut stream = self.rx.into_stream().map(|v| v + 10);

        println!("{:?} listening", &name);

        while let Some(item) = stream.next().await {
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

    tokio::time::sleep(Duration::from_secs(1)).await;

    println!("Sending messages");
    tx.send(10).unwrap();
    tx.send(20).unwrap();
    tx.send(30).unwrap();
    tx.send(40).unwrap();
    tx.send(50).unwrap();
    tx.send(60).unwrap();

    println!("huh");

    let mut s = tokio::time::interval(Duration::from_millis(1000));
    while let now = s.tick().await {
        tx.send(now.elapsed().as_secs()).unwrap();
    }

    f.await;
}
