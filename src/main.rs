mod pipeline;
mod tempo;
mod transforms;

use futures::{future::select_all, StreamExt};
use pipeline::Pipeline;
use std::time::Duration;
use transforms::{ArpeggioTransform, InspectTransform};

#[derive(Debug)]
pub enum Wrapper<T> {
    Tempo,
    Value(T),
}

struct App {
    pub egress_sender: flume::Sender<Wrapper<u64>>,
    pub ingress_rx: flume::Receiver<u64>,
    pub pipelines: Vec<pipeline::Pipeline>,
}

impl App {
    pub fn new(
        egress_sender: flume::Sender<Wrapper<u64>>,
        ingress_rx: flume::Receiver<u64>,
        pipelines: Vec<pipeline::Pipeline>,
    ) -> App {
        App {
            ingress_rx,
            egress_sender,
            pipelines,
        }
    }

    pub async fn run(self) -> Option<()> {
        // Collect each pipelines' sender
        let txs: Vec<flume::Sender<Wrapper<u64>>> =
            self.pipelines.iter().map(|p| p.tx.clone()).collect::<_>();

        // Broadcast events from ingress to each pipeline sender
        tokio::spawn(async move {
            while let Ok(x) = self.ingress_rx.recv_async().await {
                txs.iter().for_each(|tx| {
                    tx.send(Wrapper::Value(x)).unwrap();
                });
            }
        });

        // Iterate through all pipelines and obtain their streams
        // Listen to all their messages and send them to the egress
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
        let _ = select_all(pipeline_futures).await;

        Some(())
    }
}

// How many threads should I use here...?
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

        loop {
            i.tick().await;
            sender.send(n).unwrap();
            n += 1;
        }
    });

    // This receiver will receive messages from all pipelines, it's up to whoever consumes this
    // To dispatch the messages over to its destination.
    while let Ok(d) = egress_receiver.recv_async().await {
        println!("{:?}", d);
    }

    f.await.unwrap();
}
