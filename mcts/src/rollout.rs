use super::{EdgeData, Game, Payoff, VertexData};
// use super::base::*;
use super::backprop;
use super::ucb;
// use super::payoff::ThudPayoff;

use ::itertools::Itertools;
use ::rand::Rng;
use ::search_graph;

use std::convert::From;
use std::error::Error;
use std::fmt;
use std::result::Result;

pub enum RolloutError<'a, G> where G: 'a + Game {
    Cycle(Vec<search_graph::nav::Edge<'a, G::State, VertexData, EdgeData<G::Statistics, G::Action>>>),
    Ucb(ucb::UcbError),
}

impl<'a, G: 'a + Game> fmt::Debug for RolloutError<'a, G> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            RolloutError::Cycle(_) => write!(f, "Cycle in path"),
            RolloutError::Ucb(ref e) => write!(f, "UCB error ({:?})", e),
        }
    }
}

impl<'a, G: 'a + Game> fmt::Display for RolloutError<'a, G> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            RolloutError::Cycle(_) => write!(f, "Cycle in path"),
            RolloutError::Ucb(ref e) => write!(f, "UCB error ({})", e),
        }
    }
}

impl<'a, G: 'a + Game> Error for RolloutError<'a, G> {
    fn description(&self) -> &str {
        match *self {
            RolloutError::Cycle(_) => "cycle",
            RolloutError::Ucb(_) => "UCB error",
        }
    }

    fn cause(&self) -> Option<&Error> {
        match *self {
            RolloutError::Ucb(ref e) => Some(e),
            _ => None,
        }
    }
}

impl<'a, G: 'a + Game> From<ucb::UcbError> for RolloutError<'a, G> {
    fn from(u: ucb::UcbError) -> RolloutError<'a, G> {
        RolloutError::Ucb(u)
    }
}

pub struct RolloutTarget<'a, G> where G: 'a + Game {
    pub node: search_graph::nav::Node<'a, G::State, VertexData, EdgeData<G::Statistics, G::Action>>,
    trace: Vec<search_graph::nav::Edge<'a, G::State, VertexData, EdgeData<G::Statistics, G::Action>>>,
}

impl<'a, G> RolloutTarget<'a, G> where G: 'a + Game {
    pub fn trace(&self) -> &[search_graph::nav::Edge<'a, G::State, VertexData, EdgeData<G::Statistics, G::Action>>] {
        &self.trace
    }
}

pub fn rollout<'a, G: 'a + Game, R: Rng>(mut node: search_graph::nav::Node<'a, G::State, VertexData, EdgeData<G::Statistics, G::Action>>,
                                         explore_bias: f64, epoch: usize, rng: &mut R)
                                         -> Result<RolloutTarget<'a, G>, RolloutError<'a, G>> {
    // Downward scan to advance state and populate downward trace.
    let mut downward_trace = Vec::new();
    {
        let mut done = false;
        while !done {
            if let Some(_) = G::Payoff::from_state(node.get_label()) {
                done = true;
            } else {
                let children = node.get_child_list();
                let best_child_index =
                    try!(ucb::find_best_child_edge_index::<G, R>(&children, epoch, explore_bias, rng));
                trace!("rollout: best child index of node {} is {}", node.get_id(), best_child_index);
                let best_child = children.get_edge(best_child_index);
                if best_child.get_data().mark_visited_in_rollout_epoch(epoch) {
                    return Err(RolloutError::Cycle(downward_trace))
                }
                done = !best_child.get_data().mark_visited();
                node = best_child.get_target();
                downward_trace.push(best_child);
            }
        }
    }
    trace!("rollout: downward_trace has edges: {}",
           downward_trace.iter().map(|e| e.get_id()).join(", "));
    // Upward scan to do best-child backprop.
    let mut upward_trace = Vec::new();
    let mut frontier: Vec<search_graph::nav::Node<'a, G::State, VertexData, EdgeData<G::Statistics, G::Action>>> =
        downward_trace.iter().map(|e| e.get_source()).collect();
    loop {
        match frontier.pop() {
            Some(n) => {
                let parents = n.get_parent_list();
                for parent in backprop::ParentSelectionIter::<G, search_graph::nav::ParentListIter<'a, G::State, VertexData, EdgeData<G::Statistics, G::Action>>>::new(parents.iter(), explore_bias, epoch) {
                    frontier.push(parent.get_source());
                    upward_trace.push(parent);
                }
            },
            _ => break,
        }
    }
    trace!("rollout: upward_trace has edges: {}",
           upward_trace.iter().map(|e| e.get_id()).join(", "));
    downward_trace.extend(upward_trace.into_iter());
    // Retain only unique edges.
    downward_trace.sort_by_key(|e| e.get_id());
    downward_trace =
        downward_trace.into_iter()
        .group_by_lazy(|e| e.get_id()).into_iter()
        .map(|(_, mut es)| es.next().unwrap())
        .collect();
    trace!("rollout: final trace has edges: {}",
           downward_trace.iter().map(|e| e.get_id()).join(", "));
    trace!("rollout: ended on node {}", node.get_id());
    Ok(RolloutTarget { node:node, trace: downward_trace, })
}
