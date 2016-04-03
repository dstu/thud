use ::mcts::base::*;
use ::mcts::ucb;

use ::search_graph::{Target, Traversal};
use ::rand::Rng;

use std::error::Error;
use std::fmt;

pub enum RolloutError<'a> {
    Cycle(SearchPath<'a>),
    Ucb(ucb::UcbError),
}

pub type Result<'a> = ::std::result::Result<
        Target<MutNode<'a>, EdgeExpander<'a>>, RolloutError<'a>>;

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

pub fn rollout<'a, R: Rng>(node: MutNode<'a>, state: &mut State, explore_bias: f64,
                           epoch: usize, rng: &mut R) -> Result<'a> {
    let mut path = SearchPath::new(node);
    loop {
        if !path.is_head_expanded() {
            match path.to_head() {
                Target::Unexpanded(edge) => {
                    let edge_id = edge.get_id();
                    let source_id = edge.get_source().get_id();
                    match edge.to_target() {
                        Target::Unexpanded(expander) => {
                            trace!("rollout: ended on edge {} (from node {})", edge_id, source_id);
                            return Ok(Target::Unexpanded(expander))
                        },
                        _ => panic!("unexpanded search path head resolves to expanded edge"),
                    }
                },
                _ => panic!("search path head should not be expanded but resolves as such"),
            }
        }

        if match path.head() {
            Target::Expanded(ref n) => n.get_data().visited_in_rollout_epoch(epoch),
            _ => panic!("expanded search path head resolves to unexpanded edge"),
        } {
            return Err(RolloutError::Cycle(path))
        }

        match path.push(|n| {
            let index = try!(ucb::find_best_child_edge_index(
                &n.get_child_list(), state.active_player().role(), epoch, explore_bias, rng));
            trace!("rollout: select child {} of node {} (edge {} with statistics {:?}), outgoing play {:?} by {:?}",
                   index, n.get_id(), n.get_child_list().get_edge(index).get_id(), n.get_child_list().get_edge(index).get_data().statistics.get(),
                   n.get_child_list().get_edge(index).get_data().action, state.active_player());
            Ok(Some(Traversal::Child(index)))
        }) {
            Ok(Some(selected_edge)) => {
                trace!("rollout: performing action {:?} by {:?}",
                       selected_edge.get_data().action, state.active_player());
                state.do_action(&selected_edge.get_data().action)
            },
            Ok(None) => panic!("rollout: failed to select a child"),
            Err(::search_graph::SearchError::SelectionError(e)) =>
                return Err(RolloutError::Ucb(e)),
            Err(e) =>
                panic!("Internal error in rollout: {}", e),
        }
    }
}
