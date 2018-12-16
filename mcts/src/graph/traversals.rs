use crate::ThreadId;

use std::clone::Clone;
use std::convert::From;
use std::default::Default;
use std::fmt;
use std::sync::atomic;
use std::usize;

#[derive(Clone, Copy, Debug)]
pub struct Traversals {
  value: usize,
}

impl Traversals {
  pub fn concurrent_traversals(&self) -> u32 {
    usize::count_ones(self.value)
  }

  pub fn traversed_in_thread(&self, thread: &ThreadId) -> bool {
    (self.value & (1 << thread.as_u8())) != 0
  }
}

impl From<usize> for Traversals {
  fn from(value: usize) -> Self {
    Traversals { value: value }
  }
}

pub struct AtomicTraversals {
  value: atomic::AtomicUsize,
}

impl AtomicTraversals {
  pub fn new() -> Self {
    AtomicTraversals {
      value: atomic::AtomicUsize::new(0),
    }
  }

  pub fn get(&self) -> Traversals {
    // TODO: do we really need Ordering::SeqCst?
    let value = self.value.load(atomic::Ordering::Acquire);
    From::from(value)
  }

  pub fn mark_traversal(&self, thread: &ThreadId) -> Traversals {
    // TODO: do we really need Ordering::SeqCst?
    let previous_value = self
      .value
      .fetch_or(1 << thread.as_u8(), atomic::Ordering::AcqRel);
    From::from(previous_value)
  }

  pub fn clear_traversal(&self, thread: &ThreadId) -> Traversals {
    // TODO: do we really need Ordering::SeqCst?
    let previous_value = self
      .value
      .fetch_and(!(1 << thread.as_u8()), atomic::Ordering::AcqRel);
    From::from(previous_value)
  }
}

impl Clone for AtomicTraversals {
  fn clone(&self) -> Self {
    // TODO: do we really need Ordering::SeqCst?
    AtomicTraversals {
      value: atomic::AtomicUsize::new(self.value.load(atomic::Ordering::Acquire)),
    }
  }
}

impl fmt::Debug for AtomicTraversals {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let data = self.get();
    write!(f, "AtomicTraversals {{ value: {:b}, }}", data.value)
  }
}

impl Default for AtomicTraversals {
  fn default() -> Self {
    AtomicTraversals {
      value: atomic::AtomicUsize::new(0),
    }
  }
}
