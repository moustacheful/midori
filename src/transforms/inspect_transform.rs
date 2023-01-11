use super::Transform;
use crate::Wrapper;

pub struct InspectTransform {
    pub prefix: String,
}

impl Transform for InspectTransform {
    fn on_message(&mut self, v: Wrapper<u64>) -> Option<Wrapper<u64>> {
        println!("[{:?}]: {:?}", self.prefix, v);

        Some(v)
    }
}
