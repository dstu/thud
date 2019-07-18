//! Interface and implementations for the rollout phase of MCTS.

use crate::SearchSettings;
use crate::graph::{EdgeData, VertexData};
use crate::game::{Game, Payoff};

use std::convert::From;
use std::error::Error;
use std::fmt;
use std::result::Result;

use log::error;
use rand::Rng;
use search_graph::nav::{ChildList, Edge, Node};

/// Error type for MCTS rollout.
pub enum RolloutError<'a, G, E>
where
  G: 'a + Game,
  E: Error,
{
  Cycle(Vec<Edge<'a, G::State, VertexData, EdgeData<G>>>),
  Selector(E),
}

impl<'a, G: 'a + Game, E: Error> fmt::Debug for RolloutError<'a, G, E> {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match *self {
      RolloutError::Cycle(_) => write!(f, "Cycle in path"),
      RolloutError::Selector(ref e) => write!(f, "Selector error ({:?})", e),
    }
  }
}

impl<'a, G: 'a + Game, E: Error> fmt::Display for RolloutError<'a, G, E> {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match *self {
      RolloutError::Cycle(_) => write!(f, "Cycle in path"),
      RolloutError::Selector(ref e) => write!(f, "Selector error ({})", e),
    }
  }
}

impl<'a, G: 'a + Game, E: Error> Error for RolloutError<'a, G, E> {
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

impl<'a, G: 'a + Game, E: Error> From<E> for RolloutError<'a, G, E> {
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
    children: ChildList<'a, G::State, VertexData, EdgeData<G>>,
    rng: &mut R,
  ) -> Result<Option<Edge<'a, G::State, VertexData, EdgeData<G>>>, Self::Error>;
}

/// Traverses the game graph downwards from `node` down to some terminating
/// vertex in the search graph. The terminating vertex will either have a known
/// payoff, or yield `None` when `selector.select()` is called on it. Returns
/// the terminating vertex, or an error.
pub fn rollout<'a, G, S, R>(
  mut node: Node<'a, G::State, VertexData, EdgeData<G>>,
  selector: S,
  rng: &mut R,
) -> Result<Node<'a, G::State, VertexData, EdgeData<G>>, RolloutError<'a, G, S::Error>>
where
  G: 'a + Game,
  S: RolloutSelector<G, R>,
  R: Rng,
{
  loop {
    if let Some(_) = G::Payoff::from_state(node.get_label()) {
      // Hit known payoff.
      break;
    } else {
      match selector.select(node.get_child_list(), rng)? {
        Some(best_child) => {
          best_child.get_data().mark_rollout_traversal();
          node = best_child.get_target();
        },
        None => {
          error!("selector declined to choose a child");
          break;
        }
      }
    }
  }
  Ok(node)
}

//     // Upward scan to do best-child backprop.
//     let mut upward_trace = Vec::new();
//     let mut frontier: Vec<search_graph::nav::Node<'a, G::State, VertexData, EdgeData<G>>> =
//         downward_trace.iter().map(|e| e.get_source()).collect();
//     loop {
//         match frontier.pop() {
//             Some(n) => {
//                 let parents = n.get_parent_list();
//                 for parent in backprop::ParentSelectionIter::<G, search_graph::nav::ParentListIter<'a, G::State, VertexData, EdgeData<G>>>::new(parents.iter(), explore_bias, epoch) {
//                     frontier.push(parent.get_source());
//                     upward_trace.push(parent);
//                 }
//             },
//             _ => break,
//         }
//     }
//     trace!("rollout: upward_trace has edges: {}",
//            upward_trace.iter().map(|e| e.get_id()).join(", "));
//     downward_trace.extend(upward_trace.into_iter());
//     // Retain only unique edges.
//     downward_trace.sort_by_key(|e| e.get_id());
//     downward_trace =
//         downward_trace.into_iter()
//         .group_by_lazy(|e| e.get_id()).into_iter()
//         .map(|(_, mut es)| es.next().unwrap())
//         .collect();
//     trace!("rollout: final trace has edges: {}",
//            downward_trace.iter().map(|e| e.get_id()).join(", "));
//     trace!("rollout: ended on node {}", node.get_id());
//     Ok(RolloutTarget { node:node, trace: downward_trace, })
// }
