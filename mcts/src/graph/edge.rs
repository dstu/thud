use crate::Game;

use std::clone::Clone;
use std::default::Default;
use std::sync::atomic;

#[derive(Debug)]
pub struct EdgeData<G>
where
  G: Game,
{
  /// The game action that this edge represents.
  action: G::Action,
  /// Statistics for payoffs that resulted from taking this edge's action.
  pub statistics: G::Statistics,
  /// Tracks:
  ///
  /// * Whether an edge has ever been traversed. Default false. Set to true when
  ///   edge is first traversed and attached to a known game state.
  /// * Whether an edge has been traversed in rollout. Default false. Set to
  ///   true when visited during rollout. Set to false when visited during
  ///   backprop.
  /// * Whether an edge has been traversed in backprop. Default false. Set to
  ///   true when visited during backprop. Set to false when visited during
  ///   rollout.
  fields: atomic::AtomicUsize,
}

impl<G> Clone for EdgeData<G>
where
  G: Game,
{
  fn clone(&self) -> Self {
    EdgeData {
      action: self.action.clone(),
      statistics: self.statistics.clone(),
      fields: atomic::AtomicUsize::new(self.fields.load(atomic::Ordering::SeqCst)),
    }
  }
}

impl<G> EdgeData<G>
where
  G: Game,
{
  /// Creates a new edge data item that corresponds to a given game action.
  pub fn new(action: G::Action) -> Self {
    EdgeData {
      action: action,
      statistics: Default::default(),
      fields: atomic::AtomicUsize::new(0),
    }
  }

  /// Returns the game action that this edge corresponds to.
  pub fn action(&self) -> &G::Action {
    &self.action
  }

  /// Marks the edge as having been traversed at least once (and attached to a
  /// known game state). Returns the prior value of this field.
  pub fn mark_traversal(&self) -> bool {
    (self.fields.fetch_and(0b111, atomic::Ordering::SeqCst) & 0b001) != 0
  }

  /// Marks the edge as having been traversed in rollout. Clears the backprop
  /// traversal bit. Returns the prior value of this field.
  pub fn mark_rollout_traversal(&self) -> bool {
    (self.fields.fetch_and(0b011, atomic::Ordering::SeqCst) & 0b010) != 0
  }

  /// Marks the edge as having been traversed in backprop. Clears the rollout
  /// traversal bit. Returns the prior value of this field.
  pub fn mark_backprop_traversal(&self) -> bool {
    (self.fields.fetch_and(0b101, atomic::Ordering::SeqCst) & 0b100) != 0
  }
}
