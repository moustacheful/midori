use super::Transform;
use crate::{
    midi_event::{Controller, MIDIEvent, MIDIRouterEvent, NoteEvent},
    scheduler::SchedulerHandler,
};
use schemars::JsonSchema;
use serde::Deserialize;
use std::fs;
use wasmer::{imports, Function, Instance, Module, Store, Value};

#[derive(Debug, Deserialize, JsonSchema)]
pub struct WasmTransformOptions {
    path: String,
}

pub struct WasmTransform {
    module_path: String,
    module_instance: Option<Instance>,
    module_store: Option<Store>,
    scheduler: Option<SchedulerHandler>,
}

impl WasmTransform {
    pub fn from_config(options: WasmTransformOptions) -> Self {
        Self {
            module_path: options.path,
            module_instance: None,
            module_store: None,
            scheduler: None,
        }
    }

    fn setup_wasm(&mut self) {
        if self.scheduler.is_none() {
            println!("No scheduler available on WasmTransform");
            return;
        }
        // Use the set scheduler so we can send messages to it via WASM
        let scheduler = self.scheduler.as_ref().unwrap();

        // Wasm setup
        let binary = fs::read(self.module_path.clone()).unwrap();
        let mut store = Store::default();
        let module = Module::new(&store, &binary).unwrap();

        // Prepare the import object to be passed to the wasm instance
        let scheduler_clone = scheduler.clone();
        let import_object = imports! {
            "index" => {
                "$sendLater" => Function::new_typed(&mut store, move |a: i32, b: i32, c: i32, d: i32, delay: i32| {
                    if let Some(event) = values_to_midi_event(a,b,c,d) {

                        scheduler_clone.send_later(
                            MIDIRouterEvent {
                                device: "wasm".to_string(),
                                event,
                            },
                            delay as u64,
                        );

                    } else {
                        println!("Ignored because couldn't transform to midievent")
                    }
                }),
            }
        };

        self.module_instance = Some(Instance::new(&mut store, &module, &import_object).unwrap());
        self.module_store = Some(store);
    }
}

impl Transform for WasmTransform {
    fn get_tempo_subdiv(&self) -> Option<f64> {
        Some(0.50)
    }
    fn set_scheduler(&mut self, scheduler: SchedulerHandler) {
        self.scheduler = Some(scheduler);
        self.setup_wasm();
    }

    fn on_tick(&mut self, _scheduler: &SchedulerHandler) -> Option<MIDIRouterEvent> {
        let on_tick = self
            .module_instance
            .as_ref()
            .unwrap()
            .exports
            .get_function("onTick")
            .unwrap();

        on_tick
            .call(&mut self.module_store.as_mut().unwrap(), &[])
            .unwrap();

        None
    }

    fn on_message(
        &mut self,
        v: MIDIRouterEvent,
        _scheduler: &SchedulerHandler,
    ) -> Option<MIDIRouterEvent> {
        let on_message = self
            .module_instance
            .as_ref()
            .unwrap()
            .exports
            .get_function("onMessage")
            .unwrap();

        let args = midi_event_to_values(v.event);

        on_message
            .call(&mut self.module_store.as_mut().unwrap(), &args)
            .unwrap();

        None
    }
}

fn values_to_midi_event(m: i32, v1: i32, v2: i32, v3: i32) -> Option<MIDIEvent> {
    let [local_v1, local_v2, local_v3]: [u8; 3] = [v1, v2, v3].map(|v| v.try_into().unwrap_or(0));

    let message_code = m;
    match message_code {
        0 => Some(MIDIEvent::NoteOn({
            NoteEvent {
                channel: local_v1,
                note: local_v2,
                velocity: local_v3,
            }
        })),

        1 => Some(MIDIEvent::NoteOff({
            NoteEvent {
                channel: local_v1,
                note: local_v2,
                velocity: 0,
            }
        })),

        3 => Some(MIDIEvent::Controller(crate::midi_event::Controller {
            channel: local_v1,
            controller: local_v2,
            value: local_v3,
        })),

        _ => None,
    }
}

fn midi_event_to_values(m: MIDIEvent) -> [Value; 4] {
    dbg!(&m);
    match m {
        MIDIEvent::NoteOff(NoteEvent {
            channel,
            note,
            velocity,
        }) => [
            Value::I32(1),
            Value::I32(channel as i32),
            Value::I32(note as i32),
            Value::I32(velocity as i32),
        ],
        MIDIEvent::NoteOn(NoteEvent {
            channel,
            note,
            velocity,
        }) => [
            Value::I32(0),
            Value::I32(channel as i32),
            Value::I32(note as i32),
            Value::I32(velocity as i32),
        ],
        MIDIEvent::Controller(Controller {
            channel,
            controller,
            value,
        }) => [
            Value::I32(3),
            Value::I32(channel as i32),
            Value::I32(controller as i32),
            Value::I32(value as i32),
        ],
        MIDIEvent::PolyphonicPressure(_) => todo!(),
        MIDIEvent::ChannelPressure(_) => todo!(),
        MIDIEvent::ProgramChange(_) => todo!(),
        MIDIEvent::PitchBend(_) => todo!(),
    }
}
