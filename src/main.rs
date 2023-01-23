mod app;
mod midi_event;
mod midi_mapper;
mod pipeline;
mod tempo;
mod transforms;

use app::App;
use midi_mapper::MidiMapper;
use pipeline::Pipeline;
use transforms::{
    ArpeggioTransform, DistributeTransform, FilterTransform, FilterTransformOptions, MapTransform,
    MapTransformOptions, OutputTransform,
};

#[derive(Debug)]
pub enum Wrapper<T> {
    Tempo,
    Value(T),
}

// How many threads should I use here...?
#[tokio::main(flavor = "multi_thread", worker_threads = 4)]
async fn main() {
    let pipelines = vec![
        Pipeline::new(
            "One".to_string(),
            vec![
                Box::new(FilterTransform::new(FilterTransformOptions {
                    channels: vec![0],
                })),
                Box::new(ArpeggioTransform::new(Some(8))),
                Box::new(DistributeTransform::new(vec![0, 1, 2])),
                Box::new(OutputTransform::new("modelcycles-out".to_string())),
            ],
        ),
        Pipeline::new(
            "Two".to_string(),
            vec![
                Box::new(FilterTransform::new(FilterTransformOptions {
                    channels: vec![9],
                })),
                Box::new(MapTransform::new(MapTransformOptions {
                    channels: vec![(9, 5)],
                    cc: vec![],
                })),
                Box::new(OutputTransform::new("modelcycles-out".to_string())),
            ],
        ),
        Pipeline::new(
            "Three".to_string(),
            vec![
                Box::new(FilterTransform::new(FilterTransformOptions {
                    channels: vec![8],
                })),
                Box::new(MapTransform::new(MapTransformOptions {
                    cc: vec![(74, 16)],
                    channels: vec![(8, 4)],
                })),
                Box::new(ArpeggioTransform::new(Some(2))),
                Box::new(OutputTransform::new("modelcycles-out".to_string())),
            ],
        ),
    ];

    let app = App::new(pipelines);

    let mut midi_mapper = MidiMapper::new();
    midi_mapper.add_output("Artiphon Orba".to_string(), "orba-out".to_string());
    midi_mapper.add_input("Artiphon Orba".to_string(), "orba".to_string());
    midi_mapper.add_output("Elektron".to_string(), "modelcycles-out".to_string());

    midi_mapper.start(app);
}
