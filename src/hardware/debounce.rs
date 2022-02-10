use crate::hardware::{time, TimeDuration, TimeInstant};
use heapless::LinearMap;
#[allow(unused_imports)]
use rtt_target::*;

pub type Debouncer<const LEN: usize, Id> = LinearMap<Id, TimeInstant, LEN>;

pub trait DebouncerExt<Id> {
    fn is_ok(&self, id: Id) -> bool;
    fn update(&mut self, id: Id, delay: TimeDuration);
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

    fn update(&mut self, id: Id, delay: TimeDuration) {
        if let Some(instant) = self.get_mut(&id) {
            *instant = time::now() + delay;
        } else {
            self.insert(id, time::now() + delay).ok();
        }
    }
}
