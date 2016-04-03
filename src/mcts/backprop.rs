use ::mcts::base::*;
use ::game;
use ::mcts::payoff::Payoff;
use ::mcts::ucb;

use ::search_graph;
use ::rand::Rng;

use std::collections::HashSet;
use std::cmp;
use std::iter::Iterator;

/// Iterable view over parents of a graph node, which selects for those parents
/// for which this node is a best child.
struct ParentSelectionIter<'a> {
    parents: ::std::iter::Enumerate<ParentListIter<'a>>,
    role: game::Role,
    explore_bias: f64,
}

impl<'a> ParentSelectionIter<'a> {
    pub fn new(parents: ParentList<'a>, role: game::Role, explore_bias: f64) -> Self {
        ParentSelectionIter {
            parents: parents.iter().enumerate(), role: role, explore_bias: explore_bias,
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
                    if ucb::is_best_child(e, self.role, self.explore_bias) {
                        trace!("ParentSelectionIter::next: edge {} (from node {}) is a best child", e.get_id(), e.get_source().get_id());
                        return Some(i)
                    }
                    trace!("ParentSelectionIter::next: edge {} (data {:?}) is not a best child", e.get_id(), e.get_data());
                },
            }
        }
    }
}

pub fn backprop_payoff<'a, R: Rng>(node: Node<'a>, epoch: usize, payoff: Payoff,
                                   role: game::Role, explore_bias: f64, rng: &mut R) {
    let mut to_visit = vec![(role, node)];
    while !to_visit.is_empty() {
        let (player, node) = to_visit.pop().unwrap();
        trace!("backprop_payoff: looking at moves by player {:?} incoming to node {}", player, node.get_id());
        if node.get_data().visited_in_backprop_epoch(epoch) {
            trace!("backprop_payoff: node {} already visited in epoch {}", node.get_id(), epoch);
        } else {
            let parents = node.get_parent_list();
            // Collect parent nodes into a materialized collection because
            // updating the statistics of parent edges changes their best child
            // status, and ParentSelectionIter is lazy.
            let best_parents: Vec<usize> =
                ParentSelectionIter::new(node.get_parent_list(), player, explore_bias).collect();
            trace!("backprop_payoff: found {} parents of node {} (out of {} total); player = {:?}",
                   best_parents.len(), node.get_id(), parents.len(), player);
            for i in best_parents.into_iter() {
                let e = parents.get_edge(i);
                trace!("backprop_payoff: increment_visit(edge {}, {:?})", e.get_id(), payoff);
                let mut stats = e.get_data().statistics.get();
                stats.increment_visit(payoff);
                e.get_data().statistics.set(stats);
                to_visit.push((player.toggle(), e.get_source()));
            }
            trace!("backprop_payoff: increment_visit(node {}, {:?})", node.get_id(), payoff);
            let mut stats = node.get_data().statistics.get();
            stats.increment_visit(payoff);
            node.get_data().statistics.set(stats);
        }
    }
}

pub fn backprop_known_payoff<'a>(mut node: MutNode<'a>, p: Payoff) {
    let mut visited_nodes = HashSet::new();
    node.get_data_mut().known_payoff = Some(p);
    backprop_known_payoff_recursive(node, p, &mut visited_nodes)
}

fn backprop_known_payoff_recursive<'a>(node: MutNode<'a>, p: Payoff, visited_nodes: &mut HashSet<usize>) {
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
                            Some(p) => Some(Payoff { weight: 1,
                                                     values: [cmp::max(p.values[0], known.values[0]),
                                                              cmp::max(p.values[1], known.values[1])], }),
                        },
                    },
            }
        }
    }
    node.get_data_mut().known_payoff = payoff;
    payoff.is_some()
}
