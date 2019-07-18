use std::clone::Clone;
use std::default::Default;
use std::sync::atomic;

#[derive(Debug)]
pub struct VertexData {
  /// True iff the children of this vertex have been added to the playout
  /// graph. Vertices are added in an unexpanded state.
  expanded: atomic::AtomicBool,
}

impl VertexData {
  pub fn expanded(&self) -> bool {
    self.expanded.load(atomic::Ordering::SeqCst)
  }

  pub fn mark_expanded(&self) -> bool {
    self.expanded.swap(true, atomic::Ordering::SeqCst)
  }
}

impl Clone for VertexData {
  fn clone(&self) -> Self {
    VertexData {
      expanded: atomic::AtomicBool::new(self.expanded.load(atomic::Ordering::SeqCst)),
    }
  }
}

impl Default for VertexData {
  fn default() -> Self {
    VertexData {
      expanded: atomic::AtomicBool::new(false),
    }
  }
}
