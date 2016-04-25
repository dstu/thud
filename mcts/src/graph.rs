use super::Statistics;
use std::sync::atomic;

// pub type Graph<G: Game> =
//     search_graph::Graph<G::State, VertexData, EdgeData<G::Statistics, G::Action>>;
// pub type Vertex<'a, G: 'a + Game> =
//     search_graph::nav::Node<'a, G::State, VertexData, EdgeData<G::Statistics, G::Action>>;
// pub type Edge<'a, G: 'a + Game> =
//     search_graph::nav::Edge<'a, G::State, VertexData, EdgeData<G::Statistics, G::Action>>;
// pub type ChildList<'a, G: 'a + Game> =
//     search_graph::nav::ChildList<'a, G::State, VertexData, EdgeData<G::Statistics, G::Action>>;
// pub type ChildListIter<'a, G: 'a + Game> =
//     search_graph::nav::ChildListIter<'a, G::State, VertexData, EdgeData<G::Statistics, G::Action>>;
// pub type ParentList<'a, G: 'a + Game> =
//     search_graph::nav::ParentList<'a, G::State, VertexData, EdgeData<G::Statistics, G::Action>>;
// pub type ParentListIter<'a, G: 'a + Game> =
//     search_graph::nav::ParentListIter<'a, G::State, VertexData, EdgeData<G::Statistics, G::Action>>;
// pub type MutVertex<'a, G: 'a + Game> =
//     search_graph::mutators::MutNode<'a, G::State, VertexData, EdgeData<G::Statistics, G::Action>>;

#[derive(Debug)]
pub struct VertexData {
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

#[derive(Debug)]
pub struct EdgeData<S, A> where S: Statistics {
    rollout_epoch: atomic::AtomicUsize,
    backtrace_epoch: atomic::AtomicUsize,
    visited: atomic::AtomicBool,
    pub action: A,
    pub statistics: S,
}

impl<S, A> Clone for EdgeData<S, A> where S: Statistics + Clone, A: Clone {
    fn clone(&self) -> Self {
        EdgeData {
            // TODO: do we really need Ordering::SeqCst?
            rollout_epoch: atomic::AtomicUsize::new(
                self.rollout_epoch.load(atomic::Ordering::Acquire)),
            backtrace_epoch: atomic::AtomicUsize::new(
                self.backtrace_epoch.load(atomic::Ordering::Acquire)),
            visited: atomic::AtomicBool::new(
                self.visited.load(atomic::Ordering::Acquire)),
            action: self.action.clone(),
            statistics: self.statistics.clone(),
        }
    }
}

impl<S, A> EdgeData<S, A> where S: Statistics {
    pub fn new(action: A) -> Self {
        EdgeData {
            rollout_epoch: atomic::AtomicUsize::new(0),
            backtrace_epoch: atomic::AtomicUsize::new(0),
            visited: atomic::AtomicBool::new(false),
            action: action,
            statistics: Default::default(),
        }
    }

    // Returns true iff edge was not previously marked as visited.
    pub fn mark_visited_in_rollout_epoch(&self, epoch: usize) -> bool {
        // TODO: do we really need Ordering::SeqCst?
        let previous_value = self.rollout_epoch.swap(epoch, atomic::Ordering::AcqRel);
        assert!(previous_value <= epoch,
                "Previous rollout epoch > current epoch ({} > {})", previous_value, epoch);
        previous_value >= epoch
    }

    // Returns true iff edge was not previously marked as visited.
    pub fn visited_in_backtrace_epoch(&self, epoch: usize) -> bool {
        // TODO: do we really need Ordering::SeqCst?
        let previous_value = self.backtrace_epoch.swap(epoch, atomic::Ordering::AcqRel);
        assert!(previous_value <= epoch,
                "Previous backtrace epoch > current epoch ({} > {})", previous_value, epoch);
        previous_value >= epoch
    }

    // Returns true iff previously visited.
    pub fn mark_visited(&self) -> bool {
        // TODO: do we really need Ordering::SeqCst?
        self.visited.swap(true, atomic::Ordering::AcqRel)
    }
}
