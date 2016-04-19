use super::base::*;
use super::ucb;

use ::thud_game;

use std::iter::Iterator;

/// Iterable view over parents of a graph node, which selects for those parents
/// for which this node is a best child.
struct ParentSelectionIter<'a> {
    parents: ::std::iter::Enumerate<ParentListIter<'a>>,
    role: thud_game::Role,
    explore_bias: f64,
}

impl<'a> ParentSelectionIter<'a> {
    pub fn new(parents: ParentList<'a>, role: thud_game::Role, explore_bias: f64) -> Self {
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
                    if ucb::is_best_child(e, self.explore_bias) {
                        trace!("ParentSelectionIter::next: edge {} (from node {}) is a best child", e.get_id(), e.get_source().get_id());
                        return Some(i)
                    }
                    trace!("ParentSelectionIter::next: edge {} (data {:?}) is not a best child", e.get_id(), e.get_data());
                },
            }
        }
    }
}

// pub fn backprop_payoff<'a, R: Rng>(node: Node<'a>, epoch: usize, payoff: Payoff,
//                                    role: thud_game::Role, explore_bias: f64, rng: &mut R) {
//     let mut to_visit = vec![(role, node)];
//     while !to_visit.is_empty() {
//         let (player, node) = to_visit.pop().unwrap();
//         trace!("backprop_payoff: looking at moves by player {:?} incoming to node {}", player, node.get_id());
//         if node.get_data().mark_visited_in_backprop_epoch(epoch) {
//             trace!("backprop_payoff: node {} already visited in epoch {}", node.get_id(), epoch);
//         } else {                
//             let parents = node.get_parent_list();
//             // Collect parent nodes into a materialized collection because
//             // updating the statistics of parent edges changes their best child
//             // status, and ParentSelectionIter is lazy.
//             let best_parents: Vec<usize> =
//                 ParentSelectionIter::new(node.get_parent_list(), player, explore_bias).collect();
//             trace!("backprop_payoff: found {} parents of node {} (out of {} total); player = {:?}",
//                    best_parents.len(), node.get_id(), parents.len(), player);
//             for i in best_parents.into_iter() {
//                 let e = parents.get_edge(i);
//                 trace!("backprop_payoff: increment_visit(edge {}, {:?})", e.get_id(), payoff);
//                 let mut stats = e.get_data().statistics.get();
//                 stats.increment_visit(payoff);
//                 e.get_data().statistics.set(stats);
//                 to_visit.push((player.toggle(), e.get_source()));
//             }
//         }
//     }
// }
