#![feature(reflect_marker)]

extern crate itertools;
#[macro_use] extern crate log;
extern crate rand;
extern crate search_graph;

pub mod backprop;
mod game;
mod graph;
pub mod expand;
pub mod rollout;
pub mod simulate;
pub mod ucb;

// use self::payoff::*;

pub use self::game::{Game, Payoff, State, Statistics};
pub use self::graph::{EdgeData, VertexData};

use ::rand::Rng;

use std::convert::From;
use std::error::Error;
use std::fmt;
use std::marker::PhantomData;
use std::result::Result;

#[derive(Clone, Debug)]
pub enum SearchError {
    NoRootState,
    Cycle,
    NoTerminalPayoff,
    UnexpandedInCycle,
    Ucb(ucb::UcbError),
}

#[derive(Clone, Debug)]
pub enum UcbValue {
    Select,
    Value(f64),
}

impl UcbValue {
    pub fn from_success<'a, G: Game>(success: &ucb::UcbSuccess<'a, G>) -> Self {
        match success {
            &ucb::UcbSuccess::Select(_) => UcbValue::Select,
            &ucb::UcbSuccess::Value(_, v) => UcbValue::Value(v),
        }
    }
}

#[derive(Clone, Debug)]
pub struct ActionStatistics<G> where G: Game {
    pub action: G::Action,
    pub payoff: G::Payoff,
    pub ucb: Result<UcbValue, ucb::UcbError>,
}

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

impl<'a, G: 'a + Game> From<rollout::RolloutError<'a, G>> for SearchError {
    fn from(e: rollout::RolloutError<'a, G>) -> SearchError {
        match e {
            rollout::RolloutError::Cycle(_) => panic!("cycle in rollout"),
            rollout::RolloutError::Ucb(u) => From::from(u),
        }
    }
}

impl<'a> From<ucb::UcbError> for SearchError {
    fn from(e: ucb::UcbError) -> SearchError {
        SearchError::Ucb(e)
    }
}

pub fn new_search_graph<G>() -> search_graph::Graph<G::State, VertexData, EdgeData<G::Statistics, G::Action>>
     where G: Game {
         search_graph::Graph::<G::State, VertexData, EdgeData<G::Statistics, G::Action>>::new()
     }

#[derive(Clone, Copy, Debug)]
pub struct SearchSettings {
    pub simulation_count: usize,
}

pub struct SearchState<R, G> where R: Rng, G: Game {
    epoch: usize,
    rng: R,
    explore_bias: f64,
    game_type: PhantomData<G>,
}

impl<R, G> SearchState<R, G> where R: Rng, G: Game {
    pub fn new(rng: R, explore_bias: f64) -> Self {
        SearchState {
            epoch: 1,
            rng: rng,
            explore_bias: explore_bias,
            game_type: PhantomData,
        }
    }

    pub fn initialize(&self, graph: &mut search_graph::Graph<G::State, VertexData, EdgeData<G::Statistics, G::Action>>, root_state: &G::State) {
        let node = graph.add_root(root_state.clone(), Default::default());
        if node.get_child_list().is_empty() {
            expand::expand::<G>(node);
        }
    }

    pub fn search<F>(&mut self, graph: &mut search_graph::Graph<G::State, VertexData, EdgeData<G::Statistics, G::Action>>, root_state: &G::State, mut settings_fn: F)
                     -> Result<Vec<ActionStatistics<G>>, SearchError>
        where F: FnMut(usize)-> SearchSettings {
            {
                let current_epoch = self.epoch;
                try!(self.iterate_search(root_state, graph, &settings_fn(current_epoch)));
            }
            self.epoch += 1;
            let children = graph.get_node(&root_state).unwrap().get_child_list();
            let mut child_ucb_results = ucb::child_edge_ucb_scores::<G, R>(
                &children, self.epoch, self.explore_bias, &mut self.rng).into_iter();
            let mut root_stats = Vec::new();
            for child_edge in children.iter() {
                root_stats.push(ActionStatistics {
                    action: child_edge.get_data().action.clone(),
                    payoff: child_edge.get_data().statistics.as_payoff(),
                    ucb: child_ucb_results.next().unwrap().map(|s| UcbValue::from_success(&s)),
                });
            }
            assert!(child_ucb_results.next().is_none());
            Ok(root_stats)
        }

    fn iterate_search<'a>(&mut self, state: &G::State, graph: &mut search_graph::Graph<G::State, VertexData, EdgeData<G::Statistics, G::Action>>, settings: &SearchSettings)
                          -> ::std::result::Result<(), SearchError> {
        let rollout_state_option = {
            let node: search_graph::nav::Node<G::State, VertexData, EdgeData<G::Statistics, G::Action>> = match graph.get_node(state) {
                Some(n) => n,
                None => return Err(SearchError::NoRootState),
            };
            let rollout_result: rollout::RolloutTarget<G> =
                try!(rollout::rollout(node, self.explore_bias, self.epoch, &mut self.rng));

            let rollout_to_expanded = rollout_result.node.get_data().mark_expanded();
            trace!("iterate_search: rollout_to_expanded = {}", rollout_to_expanded);
            let payoff =
                if let Some(known_payoff) = G::Payoff::from_state(rollout_result.node.get_label()) {
                    // Known payoff from rollout node.
                    trace!("rollout node {} has known payoff {:?}", rollout_result.node.get_id(), known_payoff);
                    known_payoff
                } else if rollout_to_expanded {
                    // Rollout found an unvisited edge to a node that was
                    // already expanded and has some payoff statistics. We
                    // propagate the known statistics.
                    if rollout_result.node.is_leaf() {
                        return Err(SearchError::NoTerminalPayoff)
                    }
                    trace!("iterate_search: expanded rollout node {} is expanded; propagating statistics from {} children",
                           rollout_result.node.get_id(), rollout_result.node.get_child_list().len());
                    let mut payoff = G::Payoff::default();
                    for child in rollout_result.node.get_child_list().iter() {
                        let stats: &G::Statistics = &child.get_data().statistics;
                        let child_payoff: G::Payoff = stats.as_payoff();
                        trace!("iterate_search: expanded rollout node {} child has payoff of {:?}", rollout_result.node.get_id(), child_payoff);
                        payoff += child_payoff;
                    }
                    trace!("iterate_search: expanded rollout node {} has payoff total {:?}",
                           rollout_result.node.get_id(), payoff);
                    payoff
                } else {
                    // Simulate playout from the rollout node and propagate the
                    // resulting statistics.
                    let mut payoff = G::Payoff::default();
                    let state_ref: &G::State = &rollout_result.node.get_label();
                    for _ in 0..settings.simulation_count {
                        payoff += simulate::simulate::<R, G>(&mut state_ref.clone(), &mut self.rng);
                    }
                    trace!("iterate_search: unexpanded rollout node {} gets payoff {:?}", rollout_result.node.get_id(), payoff);
                    payoff
                };

            for edge in rollout_result.trace() {
                trace!("iterate_search: backprop {:?} to edge {}", payoff, edge.get_id());
                let stats: &G::Statistics = &edge.get_data().statistics;
                stats.increment(&payoff);
            }

            if rollout_to_expanded {
                // No need to do expansion from final state.
                None
            } else {
                // The vertex for the rollout state needs to be expanded.
                let state_ref: &G::State = rollout_result.node.get_label();
                Some(state_ref.clone())
            }
        };

        if let Some(rollout_state) = rollout_state_option {
            trace!("iterate_search: rollout node needs expansion");
            expand::expand::<G>(graph.get_node_mut(&rollout_state).unwrap());
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
