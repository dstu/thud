//! Interface and implementations for the rollout phase of MCTS.

use crate::game::Game;
use crate::graph::{EdgeData, VertexData};
use crate::SearchSettings;

use std::convert::From;
use std::error::Error;
use std::fmt;
use std::result::Result;

use rand::Rng;

/// Error type for MCTS rollout.
pub enum RolloutError<G: Game, E: Error> {
  /// Rollout encountered a cycle.
  Cycle { root: G::State, elements: Vec<(G::Action, G::State)>, },
  /// The [RolloutSelector](struct.RolloutSelector.html) that
  /// [rollout](fn.rollout.html) delegates to reported some error.
  Selector(E),
}

impl<G: Game, E: Error> fmt::Debug for RolloutError<G, E> {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match *self {
      RolloutError::Cycle { ref root, ref elements, } => {
        write!(f, "Cycle in path: ")?;
        write!(f, "Start state: {:?}", root)?;
        for (action, state) in elements.iter() {
          write!(f, "; action: {:?}, state: {:?}", action, state)?;
        }
        Ok(())
      }
      RolloutError::Selector(ref e) => write!(f, "Selector error ({:?})", e),
    }
  }
}

impl<G: Game, E: Error> fmt::Display for RolloutError<G, E> {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match *self {
      RolloutError::Cycle { root: _, elements: _, } => write!(f, "Cycle in path"),
      RolloutError::Selector(ref e) => write!(f, "Selector error ({})", e),
    }
  }
}

impl<G: Game, E: Error> Error for RolloutError<G, E> {
  fn description(&self) -> &str {
    match *self {
      RolloutError::Cycle { root: _, elements: _, } => "Cycle",
      RolloutError::Selector(_) => "Selector error",
    }
  }

  fn cause(&self) -> Option<&dyn Error> {
    match *self {
      RolloutError::Selector(ref e) => Some(e),
      _ => None,
    }
  }
}

impl<G: Game, E: Error> From<E> for RolloutError<G, E> {
  fn from(e: E) -> Self {
    RolloutError::Selector(e)
  }
}

/// Provides a method for selecting an outgoing child edge to follow during
/// the rollout phase of MCTS.
pub trait RolloutSelector: for<'a> From<&'a SearchSettings> {
  type Error: Error;

  /// Returns the element of `children` that should be followed, or an error.
  fn select<'a, 'id, G: Game, R: Rng>(
    &self,
    graph: &search_graph::view::View<'a, 'id, G::State, VertexData, EdgeData<G>>,
    parent: search_graph::view::NodeRef<'id>,
    rng: &mut R,
  ) -> Result<search_graph::view::EdgeRef<'id>, Self::Error>;
}

/// Traverses the game graph downwards from `node` down to some terminating
/// vertex in the search graph.
///
/// The terminating vertex will either have a known payoff, or yield `None` when
/// `selector.select()` is called on it. Returns the path to terminating vertex,
/// whose last element is the terminating vertex, or an error.
///
/// Selection will be done minimax-style, i.e., always trying to maximize the
/// score for the currently active player.
pub fn rollout<'a, 'id, G, S, R>(
  graph: &search_graph::view::View<'a, 'id, G::State, VertexData, EdgeData<G>>,
  mut node: search_graph::view::NodeRef<'id>,
  selector: S,
  rng: &mut R,
) -> Result<search_graph::view::NodeRef<'id>, RolloutError<G, S::Error>>
where
  G: Game,
  S: RolloutSelector,
  R: Rng,
{
  loop {
    if let Some(_) = G::payoff_of(graph.node_state(node)) {
      // Hit known payoff.
      break;
    } else if graph.child_count(node) == 0 {
      // Hit leaf in search graph.
      break;
    } else {
      let child = selector.select(graph, node, rng)?;
      graph.edge_data(child).mark_rollout_traversal();
      node = graph.edge_target(child);
    }
  }
  Ok(node)
}
