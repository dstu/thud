use ::mcts::base::*;
use ::mcts::ucb;

use ::console_ui;
use ::game;

use ::search_graph;
use ::search_graph::{Target, Traversal};
use ::rand::Rng;

use std::collections::HashSet;
use std::cmp::Ordering;
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
            RolloutError::Cycle(ref path) => write!(f, "Cycle in path"),
            RolloutError::Ucb(ref e) => write!(f, "UCB error ({:?})", e),
        }
    }
}

impl<'a> fmt::Display for RolloutError<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            RolloutError::Cycle(ref path) => write!(f, "Cycle in path"),
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

pub fn rollout<'a, R: Rng>(mut node: MutNode<'a>, state: &mut game::State, bias: f64, epoch: usize,
                           rng: &mut R) -> Result<'a> {
    let mut path = SearchPath::new(node);
    loop {
        if !path.is_head_expanded() {
            match path.to_head() {
                Target::Unexpanded(edge) =>
                    match edge.to_target() {
                        Target::Unexpanded(expander) => return Ok(Target::Unexpanded(expander)),
                        _ => panic!("unexpanded search path head resolves to expanded edge"),
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
            Ok(Some(Traversal::Child(try!(ucb::find_best_child_edge(
                &n.get_child_list(), state.active_player().marker(), epoch, bias, rng)))))
        }) {
            Err(::search_graph::SearchError::SelectionError(e)) => return Err(RolloutError::Ucb(e)),
            Err(e) => panic!("Internal error in rollout: {}", e),
            _ => (),
        }
    }
}
