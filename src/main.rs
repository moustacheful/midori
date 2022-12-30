use futures::{future::select_all, StreamExt};
use std::{sync::Arc, time::Duration};

struct Pipeline {
    name: String,
    pub tx: flume::Sender<u16>,
    pub rx: flume::Receiver<u16>,
}

impl Pipeline {
    pub fn new(name: String) -> Pipeline {
        let (tx, rx) = flume::unbounded::<u16>();

        Pipeline { tx, rx, name }
    }

    pub fn send(&self, item: u16) {
        dbg!("SENDING THE THING", &item);
        self.tx.send(item).unwrap();
    }

    pub async fn listen(self) {
        let name = self.name.clone();
        let rx = self.rx.clone();

        while let Ok(item) = rx.recv_async().await {
            println!("{:?} - by {:?}", item, &name);
        }
    }
}

struct App {
    pub tx: flume::Sender<u16>,
    pub rx: flume::Receiver<u16>,
    pub pipelines: Arc<Vec<Pipeline>>,
}

impl App {
    pub fn new(
        tx: flume::Sender<u16>,
        rx: flume::Receiver<u16>,
        pipelines: Arc<Vec<Pipeline>>,
    ) -> App {
        App { tx, rx, pipelines }
    }

    pub async fn run_broadcast(self) -> Option<()> {
        let pipelines = self.pipelines;
        let txs = pipelines.iter().map(|p| p.tx.clone());
        let rx = self.rx.clone();

        let futs = pipelines.iter().map(|p| {
            tokio::spawn(async {
                println!("{:?} listening", p.name);
                Box::pin(p.listen()).await
            })
        });

        while let Ok(item) = rx.recv_async().await {
            for tx in txs {
                tx.send(item.clone()).unwrap();
            }
        }

        select_all(futs).await;

        Some(())
    }
}

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    let (tx, mut rx) = flume::unbounded::<u16>();

    let app_tx = tx.clone();
    let app_rx = rx.clone();

    let f = tokio::spawn(async {
        let app = App::new(
            app_tx,
            app_rx,
            Arc::new(vec![
                Pipeline::new("First".to_string()),
                Pipeline::new("Last".to_string()),
            ]),
        );

        app.run_broadcast().await;

        Some(())
    });

    tokio::time::sleep(Duration::from_secs(1)).await;

    tx.send(10).unwrap();
    tx.send(20).unwrap();
    tx.send(30).unwrap();
    tx.send(40).unwrap();
    tx.send(50).unwrap();
    tx.send(60).unwrap();

    f.await;
}
