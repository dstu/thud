use std::clone::Clone;
use std::default::Default;
use std::sync::atomic;

#[derive(Debug)]
pub struct VertexData {
    /// True iff the children of this vertex have been added to the playout
    /// graph. Vertices are added in the unexpanded state.
    expanded: atomic::AtomicBool,
}

impl VertexData {
    pub fn expanded(&self) -> bool {
        self.expanded.load(atomic::Ordering::Acquire)
    }

    pub fn mark_expanded(&self) -> bool {
        // TODO: do we really need Ordering::SeqCst?
        self.expanded.swap(true, atomic::Ordering::AcqRel)
    }
}

impl Clone for VertexData {
    fn clone(&self) -> Self {
        VertexData {
            // TODO: do we really need Ordering::SeqCst?
            expanded: atomic::AtomicBool::new(
                self.expanded.load(atomic::Ordering::Acquire)),
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
