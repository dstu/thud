use ::ThreadId;

use std::clone::Clone;
use std::default::Default;
use std::sync::atomic;

#[derive(Clone, Copy, Debug)]
pub struct Traversals {
    value: usize,
}

impl Traversals {
    pub fn concurrent_traversals(&self) -> u32 {
        std::usize::count_ones(self.value)
    }

    pub fn traversed_in_thread(&self, thread: &ThreadId) -> bool {
        (self.value & (1 << thread.as_u8())) != 0
    }
}

pub struct AtomicTraversals {
    value: atomic::AtomicUsize,
}

impl AtomicTraversals {
    pub fn new() -> Self {
        AtomicTraversals { value: atomic::AtomicUsize::new(0), }
    }

    pub fn get(&self) -> Data {
        // TODO: do we really need Ordering::SeqCst?
        let value = self.value.load(atomic::Ordering::Acquire);
        Data::from_value(value)
    }

    pub fn mark_traversal(&self, thread: &ThreadId) -> Data {
        // TODO: do we really need Ordering::SeqCst?
        let previous_value = self.value.fetch_or(1 << thread.as_u8(), atomic::ordering::AcqRel);
        Data::from_value(previous_value)
    }

    pub fn clear_traversal(&self, thread: &ThreadId) -> Data {
        check_thread(thread);
        // TODO: do we really need Ordering::SeqCst?
        let previous_value = self.value.fetch_and(~(1 << thread.as_u8()), atomic::ordering::AcqRel);
        Data::from_value(previous_value)
    }
}

impl Clone for AtomicTraversals {
    fn clone(&self) -> Self {
        // TODO: do we really need Ordering::SeqCst?
        AtomicTraversals {
            value: self.value.load(atomic::Ordering::Acquire),
        }
    }
}

impl fmt::Debug for AtomicTraversals {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let data = self.get();
        write!(f, "AtomicTraversals { value: {:b}, }", data.value)
    }
}

impl Default for AtomicTraversals {
    fn default() -> Self {
        AtomicTraversals {
            value: atomic::AtomicUsize::new(0),
        }
    }
}
