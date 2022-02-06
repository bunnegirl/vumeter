use crate::app::monotonics as timer;
use fugit::{ExtU32, Instant};
use heapless::LinearMap;

pub type Debouncer<const SIZE: usize> = LinearMap<usize, Instant<u32, 1, 8000000>, SIZE>;

pub trait DebouncerExt {
    fn is_ok(&self, id: usize) -> bool;
    fn update(&mut self, id: usize, delay: u32);
}

impl<const SIZE: usize> DebouncerExt for Debouncer<SIZE> {
    fn is_ok(&self, id: usize) -> bool {
        if let Some(instant) = self.get(&id) {
            return *instant < timer::now();
        }

        true
    }

    fn update(&mut self, id: usize, delay: u32) {
        if let Some(instant) = self.get_mut(&id) {
            *instant = timer::now() + delay.millis();
        } else {
            self.insert(id, timer::now() + delay.millis()).ok();
        }
    }
}
