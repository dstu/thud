use ::rand::Rng;

use super::base::*;
use super::ucb;
use super::payoff::payoff;

use std::convert::From;
use std::error::Error;
use std::fmt;

pub enum RolloutError<'a> {
    Cycle(Vec<Edge<'a>>),
    Ucb(ucb::UcbError),
}

pub type Result<'a> = ::std::result::Result<(Node<'a>, Vec<Edge<'a>>), RolloutError<'a>>;

impl<'a> fmt::Debug for RolloutError<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            RolloutError::Cycle(_) => write!(f, "Cycle in path"),
            RolloutError::Ucb(ref e) => write!(f, "UCB error ({:?})", e),
        }
    }
}

impl<'a> fmt::Display for RolloutError<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            RolloutError::Cycle(_) => write!(f, "Cycle in path"),
            RolloutError::Ucb(ref e) => write!(f, "UCB error ({})", e),
        }
    }
}

impl<'a> Error for RolloutError<'a> {
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

impl<'a> From<ucb::UcbError> for RolloutError<'a> {
    fn from(u: ucb::UcbError) -> RolloutError<'a> {
        RolloutError::Ucb(u)
    }
}

pub fn rollout<'a, R: Rng>(mut node: Node<'a>, explore_bias: f64, epoch: usize, rng: &mut R) -> Result<'a> {
    // Downward scan to advance state and populate downward trace.
    let mut downward_trace = Vec::new();
    {
        let mut done = false;
        while !done {
            if let Some(payoff) = payoff(node.get_label()) {
                done = true;
            } else {
                let children = node.get_child_list();
                let best_child_index =
                    try!(ucb::find_best_child_edge_index(&children, epoch, explore_bias, rng));
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
    trace!("rollout: downward_trace has {} elements", downward_trace.len());
    // Upward scan to do best-child backprop.
    let mut upward_trace = Vec::new();
    let mut frontier: Vec<Node<'a>> =
        downward_trace.iter().map(|e| e.get_source()).collect();
    loop {
        match frontier.pop() {
            Some(n) => {
                for parent in n.get_parent_list().iter() {
                    if !parent.get_data().visited_in_backtrace_epoch(epoch) {
                        if ucb::is_best_child(&parent, explore_bias) {
                            trace!("rollout: node {} is best child of parent node {}",
                                   n.get_id(), parent.get_id());
                            frontier.push(parent.get_source());
                            upward_trace.push(parent);
                        }
                    }
                }
            },
            _ => break,
        }
    }
    trace!("rollout: upward_trace has {} elements", upward_trace.len());
    downward_trace.extend(upward_trace.into_iter());
    trace!("rollout: ended on node {}", node.get_id());
    Ok((node, downward_trace))
}
