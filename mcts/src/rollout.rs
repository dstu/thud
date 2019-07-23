//! Interface and implementations for the rollout phase of MCTS.

use crate::game::{Game, Payoff};
use crate::graph::{EdgeData, VertexData};
use crate::SearchSettings;

use std::convert::From;
use std::error::Error;
use std::fmt;
use std::result::Result;

use log::error;
use rand::Rng;

/// Error type for MCTS rollout.
pub enum RolloutError<'a, E: Error> {
  /// Rollout encountered a cycle.
  Cycle(Vec<search_graph::view::EdgeRef<'a>>),
  /// The `RolloutSelector` that `rollout` delegates to reported some error.
  Selector(E),
}

impl<'a, E: Error> fmt::Debug for RolloutError<'a, E> {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match *self {
      RolloutError::Cycle(_) => write!(f, "Cycle in path"),
      RolloutError::Selector(ref e) => write!(f, "Selector error ({:?})", e),
    }
  }
}

impl<'a, E: Error> fmt::Display for RolloutError<'a, E> {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match *self {
      RolloutError::Cycle(_) => write!(f, "Cycle in path"),
      RolloutError::Selector(ref e) => write!(f, "Selector error ({})", e),
    }
  }
}

impl<'a, E: Error> Error for RolloutError<'a, E> {
  fn description(&self) -> &str {
    match *self {
      RolloutError::Cycle(_) => "Cycle",
      RolloutError::Selector(_) => "Selector error",
    }
  }

  fn cause(&self) -> Option<&Error> {
    match *self {
      RolloutError::Selector(ref e) => Some(e),
      _ => None,
    }
  }
}

impl<'a, E: Error> From<E> for RolloutError<'a, E> {
  fn from(e: E) -> Self {
    RolloutError::Selector(e)
  }
}

/// Provides a method for selecting an outgoing child edge to follow during
/// the rollout phase of MCTS.
pub trait RolloutSelector<G, R>: for<'a> From<&'a SearchSettings>
where
  G: Game,
  R: Rng,
{
  type Error: Error;

  /// Returns the element of `children` that should be followed, or an error.
  fn select<'a>(
    &self,
    graph: &search_graph::view::View<'a, G::State, VertexData, EdgeData<G>>,
    children: impl Iterator<Item = search_graph::view::EdgeRef<'a>>,
    rng: &mut R,
  ) -> Result<Option<search_graph::view::EdgeRef<'a>>, Self::Error>;
}

/// Traverses the game graph downwards from `node` down to some terminating
/// vertex in the search graph. The terminating vertex will either have a known
/// payoff, or yield `None` when `selector.select()` is called on it. Returns
/// the terminating vertex, or an error.
pub fn rollout<'a, G, S, R>(
  graph: &search_graph::view::View<'a, G::State, VertexData, EdgeData<G>>,
  mut node: search_graph::view::NodeRef<'a>,
  selector: S,
  rng: &mut R,
) -> Result<search_graph::view::NodeRef<'a>, RolloutError<'a, S::Error>>
where
  G: 'a + Game,
  S: RolloutSelector<G, R>,
  R: Rng,
{
  loop {
    if let Some(_) = G::Payoff::from_state(graph.node_state(node)) {
      // Hit known payoff.
      break;
    } else {
      match selector.select(graph, graph.children(node), rng)? {
        Some(best_child) => {
          graph.edge_data(best_child).mark_rollout_traversal();
          node = graph.edge_target(best_child);
        }
        None => {
          error!("selector declined to choose a child");
          break;
        }
      }
    }
  }
  Ok(node)
}
