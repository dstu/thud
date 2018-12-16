//! Utility module for expapnsion of game state at a graph node.

use super::{EdgeData, Game, State, VertexData};
use log::trace;
use search_graph;

use std::collections::HashSet;
use std::default::Default;

pub fn expand<'a, G>(
  mut node: search_graph::mutators::MutNode<'a, G::State, VertexData, EdgeData<G>>,
) where
  G: 'a + Game,
{
  let state = node.get_label().clone();
  trace!(
    "expand: expanding moves at node {} for {:?}",
    node.get_id(),
    state.active_player()
  );
  let mut child_states = HashSet::new();
  {
    let mut children = node.get_child_list_mut();
    state.for_actions(|a| {
      let mut next_state = state.clone();
      next_state.do_action(&a);
      if child_states.insert(next_state.clone()) {
        children.add_child(next_state, Default::default, EdgeData::new(a));
      }
      true
    });
  }
  node.get_data().mark_expanded();
  trace!(
    "expand: added {} children (now {} total)",
    child_states.len(),
    node.get_child_list().len()
  );
}
