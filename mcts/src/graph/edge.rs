use crate::{Game, ThreadId};

use std::clone::Clone;
use std::default::Default;
use std::sync::atomic;

use super::{AtomicTraversals, Traversals};

#[derive(Debug)]
pub struct EdgeData<G>
where
  G: Game,
{
  /// Marks whether an edge has ever been traversed. true iff this edge has
  /// been traversed during rollout by any thread, in any epoch.
  traversed: atomic::AtomicBool,
  /// Marks rollout traversal within an epoch. Fields are cleared when
  /// visiting during backprop.
  rollout_traversals: AtomicTraversals,
  /// Marks backprop traversal within an epoch. Fields are cleared when
  /// visiting during rollout.
  backprop_traversals: AtomicTraversals,
  /// The game action that this edge represents.
  action: G::Action,
  /// Statistics for payoffs that resulted from taking this edge's action.
  pub statistics: G::Statistics,
}

impl<G> Clone for EdgeData<G>
where
  G: Game,
{
  fn clone(&self) -> Self {
    EdgeData {
      // TODO: do we really need Ordering::SeqCst?
      traversed: atomic::AtomicBool::new(self.traversed.load(atomic::Ordering::Acquire)),
      rollout_traversals: self.rollout_traversals.clone(),
      backprop_traversals: self.backprop_traversals.clone(),
      action: self.action.clone(),
      statistics: self.statistics.clone(),
    }
  }
}

impl<G> EdgeData<G>
where
  G: Game,
{
  pub fn new(action: G::Action) -> Self {
    EdgeData {
      traversed: atomic::AtomicBool::new(false),
      rollout_traversals: Default::default(),
      backprop_traversals: Default::default(),
      action: action,
      statistics: Default::default(),
    }
  }

  pub fn action(&self) -> &G::Action {
    &self.action
  }

  pub fn traversed(&self) -> bool {
    // TODO: do we really need Ordering::SeqCst?
    // TODO: maybe a race condition with how this is read and set.
    self.traversed.load(atomic::Ordering::Acquire)
  }

  // /// Potential race condition with `mark_rollout_traversal`.
  // pub fn rollout_traversals(&self) -> Traversals {
  //     // TODO: do we really need Ordering::SeqCst?
  //     self.traversed.load(atomic::Ordering::Release)
  // }

  // Returns true iff edge was never previously visited in this epoch.
  pub fn mark_rollout_traversal(&self, thread: &ThreadId) -> Traversals {
    // TODO: do we really need Ordering::SeqCst?
    // TODO: maybe a race condition with how this is read and set.
    self.traversed.store(true, atomic::Ordering::Release);
    self.backprop_traversals.clear_traversal(thread);
    self.rollout_traversals.mark_traversal(thread)
  }

  pub fn mark_backprop_traversal(&self, thread: &ThreadId) -> Traversals {
    self.rollout_traversals.clear_traversal(thread);
    self.backprop_traversals.mark_traversal(thread)
  }
}
