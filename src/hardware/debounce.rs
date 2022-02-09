use crate::hardware::time;
use fugit::{ExtU32, Instant};
use heapless::LinearMap;

pub type Debouncer<const LEN: usize, Id> = LinearMap<Id, Instant<u32, 1, 8000000>, LEN>;

pub trait DebouncerExt<Id> {
    fn is_ok(&self, id: Id) -> bool;
    fn update(&mut self, id: Id, delay: u32);
}

impl<const LEN: usize, Id> DebouncerExt<Id> for Debouncer<LEN, Id>
where
    Id: core::cmp::Eq,
{
    fn is_ok(&self, id: Id) -> bool {
        if let Some(instant) = self.get(&id) {
            return *instant < time::now();
        }

        true
    }

    fn update(&mut self, id: Id, delay: u32) {
        if let Some(instant) = self.get_mut(&id) {
            *instant = time::now() + delay.millis();
        } else {
            self.insert(id, time::now() + delay.millis()).ok();
        }
    }
}
