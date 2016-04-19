#[macro_use] extern crate log;
extern crate rand;
extern crate search_graph;
extern crate syncbox;
extern crate thud_game;

pub mod base;
pub mod backprop;
pub mod expand;
pub mod payoff;
pub mod rollout;
pub mod simulate;
mod statistics;
pub mod ucb;

use self::backprop::*;
use self::payoff::*;

pub use self::base::*;
pub use self::statistics::*;

use ::rand::Rng;
use ::thud_game as game;

use std::convert::From;
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

#[derive(Debug)]
pub enum UcbProxy {
    Select,
    Value(f64),
}

impl UcbProxy {
    pub fn from_success<'a>(success: &ucb::UcbSuccess<'a>) -> Self {
        match success {
            &ucb::UcbSuccess::Select(_) => UcbProxy::Select,
            &ucb::UcbSuccess::Value(_, v) => UcbProxy::Value(v),
        }
    }
}

pub type ActionStatistics = Vec<(game::Action, Payoff, ::std::result::Result<UcbProxy, ucb::UcbError>)>;

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

impl<'a> From<rollout::RolloutError<'a>> for SearchError {
    fn from(e: rollout::RolloutError) -> SearchError {
        match e {
            rollout::RolloutError::Cycle(trace) => panic!("cycle in rollout"),
            rollout::RolloutError::Ucb(u) => From::from(u),
        }
    }
}

impl<'a> From<ucb::UcbError> for SearchError {
    fn from(e: ucb::UcbError) -> SearchError {
        SearchError::Ucb(e)
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

    pub fn search<F>(&mut self, graph: &mut Graph, root_state: &State, mut settings_fn: F) -> Result
        where F: FnMut(usize)-> SearchSettings {
            {
                let current_epoch = self.epoch;
                try!(self.iterate_search(root_state, graph, &settings_fn(current_epoch)));
            }
            self.epoch += 1;
            let children = graph.get_node(&root_state).unwrap().get_child_list();
            let mut child_ucb_results = ucb::child_edge_ucb_scores(
                &children, self.epoch, self.explore_bias, &mut self.rng).into_iter();
            let mut root_stats = Vec::new();
            for child_edge in children.iter() {
                root_stats.push((child_edge.get_data().action,
                                 child_edge.get_data().statistics.get(),
                                 child_ucb_results.next().unwrap().map(|s| UcbProxy::from_success(&s))));
            }
            assert!(child_ucb_results.next().is_none());
            Ok(root_stats)
        }

    fn iterate_search<'a>(&mut self, state: &State, graph: &mut Graph, settings: &SearchSettings)
                          -> ::std::result::Result<(), SearchError> {
        let rollout_state_option = {
            let mut node = match graph.get_node(state) {
                Some(n) => n,
                None => return Err(SearchError::NoRootState),
            };
            let (rollout_node, backprop_edges) =
                try!(rollout::rollout(node, self.explore_bias, self.epoch, &mut self.rng));

            let rollout_to_expanded = rollout_node.get_data().mark_expanded();
            trace!("iterate_search: rollout_to_expanded = {}", rollout_to_expanded);
            let payoff =
                if let Some(known_payoff) = payoff(rollout_node.get_label()) {
                    // Known payoff from rollout node.
                    trace!("rollout node {} has known payoff {:?}", rollout_node.get_id(), known_payoff);
                    known_payoff
                } else if rollout_to_expanded {
                    // Rollout found an unvisited edge to a node that was
                    // already expanded and has some payoff statistics. We
                    // propagate the known statistics.
                    if rollout_node.is_leaf() {
                        return Err(SearchError::NoTerminalPayoff)
                    }
                    trace!("iterate_search: expanded rollout node {} is expanded; propagating statistics from {} children",
                           rollout_node.get_id(), rollout_node.get_child_list().len());
                    let mut payoff = Payoff::default();
                    for child in rollout_node.get_child_list().iter() {
                        let child_payoff = child.get_data().statistics.get();
                        trace!("iterate_search: expanded rollout node {} child has payoff of {:?}", rollout_node.get_id(), child_payoff);
                        payoff += child_payoff;
                    }
                    trace!("iterate_search: expanded rollout node {} has payoff total {:?}",
                           rollout_node.get_id(), payoff);
                    payoff
                } else {
                    // Simulate playout from the rollout node and propagate the
                    // resulting statistics.
                    let mut payoff = Payoff::default();
                    let state = rollout_node.get_label().clone();
                    for _ in 0..settings.simulation_count {
                        payoff += simulate::simulate(&mut state.clone(), &mut self.rng);
                    }
                    trace!("iterate_search: unexpanded rollout node {} gets payoff {:?}", rollout_node.get_id(), payoff);
                    payoff
                };

            for edge in backprop_edges.into_iter() {
                trace!("iterate_search: backprop {:?} to edge {}", payoff, edge.get_id());
                edge.get_data().statistics.increment_visit(payoff);
            }

            if rollout_to_expanded {
                // No need to do expansion from final state.
                None
            } else {
                // The vertex for the rollout state needs to be expanded.
                Some(rollout_node.get_label().clone())
            }
        };

        if let Some(rollout_state) = rollout_state_option {
            trace!("iterate_search: rollout node needs expansion");
            expand::expand(graph.get_node_mut(&rollout_state).unwrap());
        } else {
            trace!("iterate_search: not expanding rollout node");
        }
        Ok(())
    }
}

//         match  {
//             Ok(Target::Unexpanded(expander)) => {
//                 let (expanded_node, mut role, payoff) = expand::expand(
//                     expander, state.clone(), &mut self.rng, settings.simulation_count);
//                 // The role returned by expand::expand() is the role who is
//                 // now active, so we toggle the role to see who made the move
//                 // that got to the expanded node.
//                 role = role.toggle();
//                 trace!("SearchState::iterate_search: expanded node {} with incoming move by {:?} gets payoff {:?}", expanded_node.get_id(), role, payoff);
//                 backprop_payoff(expanded_node.to_node(), self.epoch, payoff, role,
//                                 self.explore_bias, &mut self.rng);
//                 return Ok(())
//             },
//             Ok(Target::Expanded(node)) =>
//                 match payoff(&state) {
//                     None => {
//                         error!("SearchState:iterate_search: no terminal payoff for node {}", node.get_id());
//                         return Err(SearchError::NoTerminalPayoff)
//                     },
//                     Some(p) => {
//                         trace!("SearchState::iterate_search: ended on expanded node {} with payoff {:?}", node.get_id(), p);
//                         backprop_payoff(node.to_node(), self.epoch, p, state.active_role().toggle(), self.explore_bias, &mut self.rng);
//                         return Ok(())
//                     },
//                 },
//             Err(rollout::RolloutError::Cycle(stack)) => {
//                 error!("rollout found cycle:");
//                 for item in stack.iter() {
//                     match item {
//                         ::search_graph::search::StackItem::Item(e) =>
//                             error!("Edge action: {:?}", e.get_data().action),
//                         ::search_graph::search::StackItem::Head(::search_graph::Target::Expanded(n)) => {
//                             error!("Head node parent actions:");
//                             for parent in n.get_parent_list().iter() {
//                                 error!("{:?}", parent.get_data().action);
//                             }
//                             error!("Head node child actions:");
//                             for child in n.get_parent_list().iter() {
//                                 error!("{:?}", child.get_data().action);
//                             }
//                         },
//                         ::search_graph::search::StackItem::Head(::search_graph::Target::Unexpanded(e)) =>
//                             error!("Head edge action: {:?}", e.get_data().action),
//                     }
//                 }
//                 panic!("cycle in rollout")
//             },
//             Err(rollout::RolloutError::Ucb(e)) => Err(SearchError::Ucb(e)),
//             // Err(e) => panic!("{:?}", e),
//             // rollout::Result::Cycle(mut cyclic_edge) => {
            
//             //     match cyclic_edge.get_target() {
//             //         ::search_graph::Target::Expanded(child_node) =>
//             //             if child_node.get_id() == node_id {
//             //                 trace!("cycle back to root");
//             //             } else {
//             //                 trace!("cycle to intermediate vertex: {}", child_node.get_id());
//             //             },
//             //         ::search_graph::Target::Unexpanded(_) =>
//             //             return Err(SearchError::UnexpandedInCycle),
//             //     }
//             //     // We "punish" the last edge in the cycle and the vertex it
//             //     // comes from by pretending we've visited them without
//             //     // having any payoff, thereby diluting their statistics and
//             //     // discouraging a visit in the immediate future.
//             //     // TODO: This is a hack. Most problematically, it doesn't
//             //     // adequately handle the case of all paths looping back to
//             //     // root. In that case, we are stuck in a loop incrementing
//             //     // the visit count ad infinitum.
//             //     self.punish(&cyclic_edge.get_data().statistics);
//             //     self.punish(&cyclic_edge.get_source().get_data().statistics);
//             // },
//             // Err(rollout::RolloutError::Ucb(e)) => Err(SearchError::Ucb(e)),
//         }
//     }
// }

// fn punish(stats_cell: &Cell<Statistics>) {
//     let mut stats = stats_cell.get();
//     stats.visits += 1;
//     stats_cell.set(stats);
// }
