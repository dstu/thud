mod base;
mod backprop;
mod expand;
mod payoff;
mod rollout;
mod statistics;
mod ucb;

use self::backprop::*;
use self::payoff::*;

pub use self::base::*;
pub use self::statistics::*;

use ::rand::Rng;
use ::console_ui;
use ::game;
use ::search_graph::Target;

use std::cell::Cell;
use std::collections::HashSet;
use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum SearchError {
    NoRootState,
    Cycle,
    NoTerminalPayoff,
    UnexpandedInCycle,
    Ucb(ucb::UcbError),
}

pub type ActionStatistics = Vec<(game::Action, Statistics)>;

pub type Result = ::std::result::Result<ActionStatistics, SearchError>;

impl fmt::Display for SearchError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            SearchError::NoRootState => write!(f, "Root state not found"),
            SearchError::Cycle => write!(f, "Cycle encountered during rollout"),
            SearchError::NoTerminalPayoff => write!(f, "Found terminal game state with no payoff"),
            SearchError::UnexpandedInCycle =>
                write!(f, "Found cycle that included an unexpanded vertex"),
            SearchError::Ucb(ref e) => write!(f, "Error computing UCB score: {}", e),
        }
    }
}

impl Error for SearchError {
    fn description(&self) -> &str {
        match *self {
            SearchError::NoRootState => "no root state",
            SearchError::Cycle => "cycle in rollout",
            SearchError::NoTerminalPayoff => "terminal state with no payoff",
            SearchError::UnexpandedInCycle => "cycle with unexpanded vertex",
            SearchError::Ucb(ref e) => e.description(),
        }
    }

    fn cause(&self) -> Option<&Error> {
        match *self {
            SearchError::Ucb(ref e) => Some(e),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct SearchSettings {
    pub simulation_count: usize,
}

pub struct SearchState<R> where R: Rng {
    epoch: usize,
    rng: R,
    explore_bias: f64,
}

impl<R> SearchState<R> where R: Rng {
    pub fn new(rng: R, explore_bias: f64) -> Self {
        SearchState {
            epoch: 1,
            rng: rng,
            explore_bias: explore_bias,
        }
    }

    pub fn search<F>(&mut self, graph: &mut Graph, root_state: game::State, mut settings_fn: F)
                     -> Result where F: FnMut(usize)-> SearchSettings {
        match graph.get_node_mut(&root_state) {
            Some(root) => {
                trace!("SearchState::search: beginning epoch {}", self.epoch);
                let settings = settings_fn(self.epoch);
                if let Err(e) = self.iterate_search(root_state.clone(), root, &settings) {
                    return Err(e)
                }
            },
            None => return Err(SearchError::NoRootState),
        }
        self.epoch += 1;
        let mut root_stats = Vec::new();
        for child_edge in graph.get_node(&root_state).unwrap().get_child_list().iter() {
            root_stats.push((child_edge.get_data().action,
                             child_edge.get_data().statistics.get()));
        }
        Ok(root_stats)
    }

    fn iterate_search<'a>(&mut self, mut state: game::State, node: MutNode<'a>,
                          settings: &SearchSettings)
                          -> ::std::result::Result<(), SearchError> {
        match rollout::rollout(node, &mut state, self.explore_bias, self.epoch, &mut self.rng) {
            Ok(Target::Unexpanded(expander)) => {
                let (expanded_node, mut player, payoff) = expand::expand(
                    expander, state.clone(), &mut self.rng, settings.simulation_count);
                // The player returned by expand::expand() is the player who is
                // now active, so we toggle the player to see who made the move
                // that got to the expanded node.
                player.toggle();
                trace!("SearchState::iterate_search: expanded node {} with incoming move by {:?} gets payoff {:?}", expanded_node.get_id(), player, payoff);
                backprop_payoff(expanded_node.to_node(), self.epoch, payoff, player,
                                self.explore_bias, &mut self.rng);
                return Ok(())
            },
            Ok(Target::Expanded(node)) =>
                match payoff(&state) {
                    None => return Err(SearchError::NoTerminalPayoff),
                    Some(p) => {
                        backprop_known_payoff(node, p);
                        return Ok(())
                    },
                },
            Err(rollout::RolloutError::Cycle(_)) => panic!("cycle in rollout"),
            Err(rollout::RolloutError::Ucb(e)) => Err(SearchError::Ucb(e)),
            // Err(e) => panic!("{:?}", e),
            // rollout::Result::Cycle(mut cyclic_edge) => {
            
            //     match cyclic_edge.get_target() {
            //         ::search_graph::Target::Expanded(child_node) =>
            //             if child_node.get_id() == node_id {
            //                 trace!("cycle back to root");
            //             } else {
            //                 trace!("cycle to intermediate vertex: {}", child_node.get_id());
            //             },
            //         ::search_graph::Target::Unexpanded(_) =>
            //             return Err(SearchError::UnexpandedInCycle),
            //     }
            //     // We "punish" the last edge in the cycle and the vertex it
            //     // comes from by pretending we've visited them without
            //     // having any payoff, thereby diluting their statistics and
            //     // discouraging a visit in the immediate future.
            //     // TODO: This is a hack. Most problematically, it doesn't
            //     // adequately handle the case of all paths looping back to
            //     // root. In that case, we are stuck in a loop incrementing
            //     // the visit count ad infinitum.
            //     self.punish(&cyclic_edge.get_data().statistics);
            //     self.punish(&cyclic_edge.get_source().get_data().statistics);
            // },
            // Err(rollout::RolloutError::Ucb(e)) => Err(SearchError::Ucb(e)),
        }
    }
}

fn punish(stats_cell: &Cell<Statistics>) {
    let mut stats = stats_cell.get();
    stats.visits += 1;
    stats_cell.set(stats);
}
