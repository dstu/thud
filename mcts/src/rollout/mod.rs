//! Interface and implementations for the rollout phase of MCTS.

use super::{EdgeData, Game, Payoff, SearchSettings, ThreadId, VertexData};

mod error;

pub use self::error::RolloutError;

use std::convert::From;
use std::error::Error;
use std::result::Result;

use rand::Rng;
use search_graph::nav::{ChildList, Edge, Node};

pub trait RolloutSelector<G, R>: From<SearchSettings>
where
  G: Game,
  R: Rng,
{
  type Error: Error;

  fn select<'a>(
    &self,
    children: ChildList<'a, G::State, VertexData, EdgeData<G>>,
    rng: &mut R,
  ) -> Result<Option<Edge<'a, G::State, VertexData, EdgeData<G>>>, Self::Error>;
}

pub fn rollout<'a, G, S, R>(
  mut node: Node<'a, G::State, VertexData, EdgeData<G>>,
  thread: &ThreadId,
  selector: S,
  rng: &mut R,
) -> Result<Node<'a, G::State, VertexData, EdgeData<G>>, RolloutError<'a, G, S::Error>>
where
  G: 'a + Game,
  S: RolloutSelector<G, R>,
  R: Rng,
{
  let mut trace = Vec::new(); // For backtracking.
  loop {
    if let Some(_) = G::Payoff::from_state(node.get_label()) {
      break;
    } else {
      match selector.select(node.get_child_list(), rng)? {
        Some(best_child) => {
          // Selector chose a child node.
          let previous_traversals = best_child.get_data().mark_rollout_traversal(thread);
          if previous_traversals.traversed_in_thread(thread) {
            return Err(RolloutError::Cycle(trace));
          }
          node = best_child.get_target();
          trace.push(best_child);
        }
        None => {
          // Selector found no suitable child node.
          // TODO: handle this more intelligently.
          panic!("selector declined to choose a child")
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
