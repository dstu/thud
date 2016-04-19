use super::base::*;
use super::statistics::EdgeData;

use std::collections::HashSet;
use std::default::Default;

pub fn expand<'a>(mut node: MutNode<'a>) {
    let state = node.get_label().clone();
    trace!("expand: expanding moves at node {} for {:?}", node.get_id(), state.active_role());
    let mut child_states = HashSet::new();
    {
        let mut children = node.get_child_list_mut();
        for a in state.actions() {
            let mut next_state = state.clone();
            next_state.do_action(&a);
            if child_states.insert(next_state.clone()) {
                children.add_child(next_state, Default::default, EdgeData::new(a));
            }
        }
    }
    node.get_data().mark_expanded();
    trace!("expand: added {} children (now {} total)",
           child_states.len(), node.get_child_list().len());
}
