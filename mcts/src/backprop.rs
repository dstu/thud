//! Interface and implementations for backpropagation of game payoffs through a
//! search graph.

use super::{Game, EdgeData, SearchSettings, ThreadId, VertexData};

use std::error::Error;
use std::iter::Iterator;
use std::marker::PhantomData;

use ::rand::Rng;
use ::search_graph::nav::{Edge, Node, ParentList};

pub trait BackpropSelector<'a, G, R>: From<SearchSettings> where G: 'a + Game, R: Rng {
    type Error: Error;
    type Items: Iterator<Item=Edge<'a, G::State, VertexData, EdgeData<G>>>;

    fn select(&self, parents: ParentList<'a, G::State, VertexData, EdgeData<G>>,
              payoff: &G::Payoff, rng: &mut R) -> Self::Items;
}

pub fn backprop<'a, G, S, R>(node: Node<'a, G::State, VertexData, EdgeData<G>>,
                             thread: &ThreadId, payoff: &G::Payoff, selector: S, rng: &mut R)
                             -> Result<Vec<Edge<'a, G::State, VertexData, EdgeData<G>>>, <S as BackpropSelector<'a, G, R>>::Error>
    where G: Game, S: BackpropSelector<'a, G, R>, R: Rng {
    let mut stack = vec!(node);
    let mut result = Vec::new();
    loop {
        let next = stack.pop();
        match next {
            Some(node) => {
                for parent in selector.select(node.get_parent_list(), payoff, rng) {
                    let previous_traversals = parent.get_data().mark_backprop_traversal(thread);
                    if !previous_traversals.traversed_in_thread(thread) {
                        stack.push(parent.get_source());
                        result.push(parent);
                    }
                }
            },
            None => return Ok(result),
        }
    }
}

/// Iterable view over parents of a graph node, which selects for those parents
/// for which this node is a best child.
pub struct ParentSelectionIter<'a, G, I> where G: 'a + Game, I: Iterator<Item=Edge<'a, G::State, VertexData, EdgeData<G>>> {
    parents: I,
    explore_bias: f64,
    epoch: usize,
    game_type: PhantomData<&'a G>,
}

impl<'a, G, I> ParentSelectionIter<'a, G, I> where G: 'a + Game, I: Iterator<Item=Edge<'a, G::State, VertexData, EdgeData<G>>> {
    pub fn new(parents: I, explore_bias: f64, epoch: usize) -> Self {
        ParentSelectionIter {
            parents: parents,
            explore_bias: explore_bias,
            epoch: epoch,
            game_type: PhantomData,
        }
    }
}

// impl<'a, G, I> Iterator for ParentSelectionIter<'a, G, I>
//     where G: 'a + Game, I: Iterator<Item=Edge<'a, G::State, VertexData, EdgeData<G>>> {
//         type Item = Edge<'a, G::State, VertexData, EdgeData<G>>;

//         fn next(&mut self) -> Option<Edge<'a, G::State, VertexData, EdgeData<G>>> {
//             loop {
//                 match self.parents.next() {
//                     None => return None,
//                     Some(e) => {
//                         if e.get_data().visited_in_backtrace_epoch(self.epoch) {
//                             trace!("ParentSelectionIter::next: edge {} (from node {}) was already visited in backtrace epoch {}", e.get_id(), e.get_source().get_id(), self.epoch);
//                             continue
//                         }
//                         if ucb::is_best_child::<G>(&e, self.explore_bias) {
//                             trace!("ParentSelectionIter::next: edge {} (from node {}) is a best child", e.get_id(), e.get_source().get_id());
//                             return Some(e)
//                         }
//                         trace!("ParentSelectionIter::next: edge {} (data {:?}) is not a best child", e.get_id(), e.get_data());
//                     },
//                 }
//             }
//         }
//     }

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
