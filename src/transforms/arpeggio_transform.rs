use crate::Wrapper;

use super::Transform;

#[derive(Debug)]
pub struct ArpeggioTransform {
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
