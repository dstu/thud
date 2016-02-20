use ::mcts::base::*;
use ::game;
use ::mcts::payoff::Payoff;
use ::mcts::ucb;

use ::search_graph;
use ::rand::Rng;

use std::collections::HashSet;
use std::cmp;
use std::iter::Iterator;

struct ParentSelectionIter<'a> {
    parents: ::std::iter::Enumerate<ParentListIter<'a>>,
    player: game::PlayerMarker,
    explore_bias: f64,
}

impl<'a> ParentSelectionIter<'a> {
    pub fn new(parents: ParentList<'a>, mut player: game::PlayerMarker, explore_bias: f64) -> Self {
        // player.toggle();
        ParentSelectionIter {
            parents: parents.iter().enumerate(), player: player, explore_bias: explore_bias,
        }
    }
}

impl<'a> Iterator for ParentSelectionIter<'a> {
    type Item = usize;

    fn next(&mut self) -> Option<usize> {
        loop {
            match self.parents.next() {
                None => return None,
                Some((i, ref e)) => {
                    if ucb::is_best_child(e, self.player, self.explore_bias) {
                        trace!("ParentSelectionIter::next: edge {} is a best child", e.get_id());
                        return Some(i)
                    }
                    trace!("ParentSelectionIter::next: edge {} is not a best child", e.get_id());
                },
            }
        }
    }
}

pub fn backprop_payoff<'a, R: Rng>(node: MutNode<'a>, payoff: Payoff, player: game::PlayerMarker, explore_bias: f64, rng: &mut R) {
    // trace!("backprop_payoff: payoff {:?}", payoff);
    let mut visited_nodes = HashSet::new();
    backprop_payoff_recursive(node, payoff, player, explore_bias, rng, &mut visited_nodes)
}

fn backprop_payoff_recursive<'a, R: Rng>(mut node: MutNode<'a>, payoff: Payoff, mut player: game::PlayerMarker, explore_bias: f64, rng: &mut R, visited_nodes: &mut HashSet<usize>) {
    trace!("backprop_payoff_recursive: looking at node {}", node.get_id());
    let parent_edge_indices: Vec<usize> =
        ParentSelectionIter::new(node.get_parent_list(), player, explore_bias).collect();
    trace!("backprop_payoff_recursive: found {} parents of {} (out of {} total); player = {:?}", parent_edge_indices.len(), node.get_id(), node.get_parent_list().len(), player);
    trace!("backprop_payoff_recursive: increment_visit(node {})", node.get_id());
    node.get_data_mut().statistics.increment_visit(payoff);
    player.toggle();
    let mut parent_list = node.to_parent_list();
    for i in parent_edge_indices.into_iter() {
        let mut e = parent_list.get_edge_mut(i);
        trace!("backprop_payoff_recursive: increment_visit(edge {})", e.get_id());
        e.get_data_mut().statistics.increment_visit(payoff);
        let mut n = e.to_source();
        if visited_nodes.insert(n.get_id()) {
            // TODO fix potential stack exhaustion.
            backprop_payoff_recursive(n, payoff, player, explore_bias, rng, visited_nodes);
        }
    }
}

pub fn backprop_known_payoff<'a>(mut node: MutNode<'a>, p: Payoff) {
    let mut visited_nodes = HashSet::new();
    node.get_data_mut().known_payoff = Some(p);
    backprop_known_payoff_recursive(node, p, &mut visited_nodes)
}

fn backprop_known_payoff_recursive<'a>(mut node: MutNode<'a>, p: Payoff, visited_nodes: &mut HashSet<usize>) {
    let mut parents = node.to_parent_list();
    for i in 0..parents.len() {
        let mut parent = parents.get_edge_mut(i).to_source();
        if visited_nodes.insert(parent.get_id()) {
            if set_known_payoff_from_children(&mut parent) {
                // TODO self edges lead to infinite loops.
                for grandparent in parent.get_parent_list_mut().iter() {
                    visited_nodes.remove(&grandparent.get_source().get_id());
                }
                // TODO fix potential stack exhaustion.
                backprop_known_payoff_recursive(parent, p, visited_nodes);
            }
        }
    }
}

fn set_known_payoff_from_children<'a>(node: &mut MutNode<'a>) -> bool {
    let mut payoff = None;
    {
        let mut children = node.get_child_list_mut();
        for i in 0..children.len() {
            match children.get_edge_mut(i).get_target_mut() {
                search_graph::Target::Unexpanded(_) => {
                    payoff = None;
                    break
                },
                search_graph::Target::Expanded(node) =>
                    match node.get_data().known_payoff {
                        None => {
                            payoff = None;
                            break
                        },
                        Some(known) => payoff = match payoff {
                            None => Some(known),
                            // TODO: This is probably wrong; we likely want to
                            // do min/max bounds on payoff depending on whose
                            // turn it is.
                            Some(p) => Some(Payoff { values: [cmp::max(p.values[0], known.values[0]),
                                                              cmp::max(p.values[1], known.values[1])], }),
                        },
                    },
            }
        }
    }
    node.get_data_mut().known_payoff = payoff;
    payoff.is_some()
}
