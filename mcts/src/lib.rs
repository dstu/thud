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
mod search_error;
pub mod simulation;
pub mod ucb;

use self::backprop::BackpropSelector;
use self::rollout::{RolloutError, RolloutSelector};
use self::simulation::Simulator;

pub use self::game::{Game, Payoff, State, Statistics};
pub use self::graph::{EdgeData, VertexData};
pub use self::search_error::SearchError;

use std::convert::From;
use std::marker::PhantomData;
use std::mem;
use std::result::Result;

use ::rand::Rng;

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Epoch(pub u32);

impl Epoch {
    pub fn as_u32(&self) -> u32 { self.0 }

    pub fn next(&self) -> Self { Epoch(self.0 + 1) }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ThreadId(u8);

impl ThreadId {
    pub fn new(id: u8) -> Self {
        let max_thread_count = (mem::size_of::<usize>() * 8) as u8;
        assert!(id < max_thread_count,
                "Thread id {} exceeds maximum thread count {}", id, max_thread_count);
        ThreadId(id)
    }

    pub fn as_u8(&self) -> u8 { self.0 }
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

pub fn new_search_graph<G>() -> search_graph::Graph<G::State, VertexData, EdgeData<G>>
     where G: Game {
         search_graph::Graph::<G::State, VertexData, EdgeData<G>>::new()
     }

#[derive(Clone, Copy, Debug)]
pub struct SearchSettings {
    pub simulation_count: u32,
    pub explore_bias: f64,
    pub rollout_tasks: u32,
    pub simulation_tasks: u32,
}

pub struct SearchState<R, G, X, Y, Z>
    where R: Rng, G: Game, X: RolloutSelector<G, R>, Y: for<'a> BackpropSelector<'a, G, R>, Z: Simulator<G, R> {
    epoch: Epoch,
    rng: R,
    phantom_types: PhantomData<(G, X, Y, Z)>,
}

impl<R, G, X, Y, Z> SearchState<R, G, X, Y, Z>
    where R: Rng, G: Game, X: RolloutSelector<G, R>, Y: for<'a> BackpropSelector<'a, G, R>, Z: Simulator<G, R> {
    pub fn new(rng: R) -> Self {
        SearchState {
            epoch: Default::default(),
            rng: rng,
            phantom_types: PhantomData,
        }
    }

    pub fn initialize(&self, graph: &mut search_graph::Graph<G::State, VertexData,
                                                             EdgeData<G>>,
                      root_state: &G::State) {
        let node = graph.add_root(root_state.clone(), Default::default());
        if node.get_child_list().is_empty() {
            expand::expand::<G>(node);
        }
    }

    pub fn epoch(&self) -> &Epoch {
        &self.epoch
    }

    pub fn step<'a, F>(&mut self, graph: &'a mut search_graph::Graph<G::State, VertexData, EdgeData<G>>,
                       root_state: &G::State, settings: &SearchSettings)
                       -> Result<Vec<ActionStatistics<G>>, SearchError<<X as RolloutSelector<G, R>>::Error, 
                                                                       <Z as Simulator<G, R>>::Error,
                                                                       <Y as BackpropSelector<'a, G, R>>::Error>> {
        self.epoch = self.epoch.next();
        let mut expanded;
        let rollout_selector: X = From::from(*settings);
        let thread = ThreadId(0);
        let state_to_expand = {
            let rollout_target = {
                let root_node = match graph.get_node(&root_state) {
                    Some(n) => n,
                    None => return Err(SearchError::NoRootState),
                };
                match rollout::rollout(root_node, &thread, rollout_selector, &mut self.rng) {
                    Ok(n) => n,
                    Err(RolloutError::Cycle(_)) => panic!("Cycle in rollout"),
                    Err(RolloutError::Selector(e)) => return Err(SearchError::Rollout(e)),
                }
            };
            expanded = rollout_target.get_data().mark_expanded();
            let rollout_state: G::State = rollout_target.get_label().clone();
            let payoff =
                if let Some(payoff) = G::Payoff::from_state(&rollout_state) {
                    // Known payoff from rollout target.
                    payoff
                } else if expanded {
                    // Rollout found an unvisited edge to a node that was
                    // already expanded and has some payoff statistics. We
                    // propagate the known statistics from this node's children.
                    let statistics = G::Statistics::default();
                    for child in rollout_target.get_child_list().iter() {
                        statistics.increment(&child.get_data().statistics.as_payoff());
                    }
                    statistics.as_payoff()
                } else {
                    // Rollout found an unexpanded node with no known payoff, so
                    // we simulate a payoff.
                    let simulator: Z = From::from(*settings);
                    match simulator.simulate(rollout_state.clone(), &mut self.rng) {
                        Ok(p) => p,
                        Err(e) => return Err(SearchError::Simulator(e)),
                    }
                };
            let backprop_selector: Y = From::from(*settings);
            let backprop_targets =
                match backprop::backprop(rollout_target, &thread, &payoff, backprop_selector,
                                         &mut self.rng) {
                    Ok(ts) => ts,
                    Err(e) => panic!("backprop error: {}", e),  // return Err(SearchError::Backprop(e)),
                };
            for edge in backprop_targets.into_iter() {
                edge.get_data().statistics.increment(&payoff);
            }
            rollout_state
        };
        if !expanded {
            expand::expand::<G>(graph.get_node_mut(&state_to_expand).unwrap());
        }

        // Gather statistics of each child of root.
        let children = graph.get_node(&root_state).unwrap().get_child_list();
        let mut child_ucb_results = ucb::child_edge_ucb_scores::<G, R>(
            &children, self.epoch, settings.explore_bias, &mut self.rng).into_iter();
        let mut root_stats = Vec::new();
        for child_edge in children.iter() {
            root_stats.push(ActionStatistics {
                action: child_edge.get_data().action().clone(),
                payoff: child_edge.get_data().statistics.as_payoff(),
                ucb: child_ucb_results.next().unwrap().map(|s| UcbValue::from_success(&s)),
            });
        }
        assert!(child_ucb_results.next().is_none());
        Ok(root_stats)
    }

    // fn iterate_search<'a>(&mut self, state: &G::State,
    //                       graph: &mut search_graph::Graph<G::State, VertexData,
    //                                                       EdgeData<G>>,
    //                       settings: &SearchSettings)
    //                       -> ::std::result::Result<(), SearchError> {
    //     let rollout_state_option = {
    //         let node: search_graph::nav::Node<G::State, VertexData, EdgeData<G>> = match graph.get_node(state) {
    //             Some(n) => n,
    //             None => return Err(SearchError::NoRootState),
    //         };
    //         let rollout_result: rollout::RolloutTarget<G> =
    //             try!(rollout::rollout(node, self.explore_bias, self.epoch, &mut self.rng));

    //         let rollout_to_expanded = rollout_result.node.get_data().mark_expanded();
    //         trace!("iterate_search: rollout_to_expanded = {}", rollout_to_expanded);
    //         let payoff =
    //             if let Some(known_payoff) = G::Payoff::from_state(rollout_result.node.get_label()) {
    //                 // Known payoff from rollout node.
    //                 trace!("rollout node {} has known payoff {:?}", rollout_result.node.get_id(), known_payoff);
    //                 known_payoff
    //             } else if rollout_to_expanded {
    //                 // Rollout found an unvisited edge to a node that was
    //                 // already expanded and has some payoff statistics. We
    //                 // propagate the known statistics.
    //                 if rollout_result.node.is_leaf() {
    //                     return Err(SearchError::NoTerminalPayoff)
    //                 }
    //                 trace!("iterate_search: expanded rollout node {} is expanded; propagating statistics from {} children",
    //                        rollout_result.node.get_id(), rollout_result.node.get_child_list().len());
    //                 let mut payoff = G::Payoff::default();
    //                 for child in rollout_result.node.get_child_list().iter() {
    //                     let stats: &G::Statistics = &child.get_data().statistics;
    //                     let child_payoff: G::Payoff = stats.as_payoff();
    //                     trace!("iterate_search: expanded rollout node {} child has payoff of {:?}", rollout_result.node.get_id(), child_payoff);
    //                     payoff += child_payoff;
    //                 }
    //                 trace!("iterate_search: expanded rollout node {} has payoff total {:?}",
    //                        rollout_result.node.get_id(), payoff);
    //                 payoff
    //             } else {
    //                 // Simulate playout from the rollout node and propagate the
    //                 // resulting statistics.
    //                 let mut payoff = G::Payoff::default();
    //                 let state_ref: &G::State = &rollout_result.node.get_label();
    //                 for _ in 0..settings.simulation_count {
    //                     payoff += simulate::simulate::<R, G>(&mut state_ref.clone(), &mut self.rng);
    //                 }
    //                 trace!("iterate_search: unexpanded rollout node {} gets payoff {:?}", rollout_result.node.get_id(), payoff);
    //                 payoff
    //             };

    //         for edge in rollout_result.trace() {
    //             trace!("iterate_search: backprop {:?} to edge {}", payoff, edge.get_id());
    //             let stats: &G::Statistics = &edge.get_data().statistics;
    //             stats.increment(&payoff);
    //         }

    //         if rollout_to_expanded {
    //             // No need to do expansion from final state.
    //             None
    //         } else {
    //             // The vertex for the rollout state needs to be expanded.
    //             let state_ref: &G::State = rollout_result.node.get_label();
    //             Some(state_ref.clone())
    //         }
    //     };

    //     if let Some(rollout_state) = rollout_state_option {
    //         trace!("iterate_search: rollout node needs expansion");
    //         expand::expand::<G>(graph.get_node_mut(&rollout_state).unwrap());
    //     } else {
    //         trace!("iterate_search: not expanding rollout node");
    //     }
    //     Ok(())
    // }
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
