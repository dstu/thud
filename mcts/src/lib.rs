//! Single-threaded Monte Carlo tree search on directed acyclic graphs.

pub mod backprop;
pub mod expand;
mod game;
mod graph;
pub mod rollout;
mod search_error;
pub mod simulation;
pub mod ucb;

use self::backprop::BackpropSelector;
use self::rollout::RolloutSelector;
use self::simulation::Simulator;

pub use self::game::{Game, Payoff, State, Statistics};
pub use self::graph::{EdgeData, VertexData};
pub use self::search_error::SearchError;

use std::convert::From;
use std::marker::PhantomData;
use std::result::Result;

use rand::Rng;
use search_graph::nav::Node;

/// Wraps a decision made by the UCB rollout policy.
///
/// This is distinct from other types in the `ucb` module to provide a
/// representation of the decision of UCB rollout that is not bound to the
/// lifetime of a `search_graph` structure.
#[derive(Clone, Debug)]
pub enum UcbValue {
  /// Select a game state because it has not yet been explored (and so no
  /// finite UCB policy value is available).
  Select,
  /// Select a game state with the given UCB policy value.
  Value(f64),
}

impl<'a, G: Game> From<&ucb::UcbSuccess<'a, G>> for UcbValue {
  fn from(success: &ucb::UcbSuccess<'a, G>) -> Self {
    match *success {
      ucb::UcbSuccess::Select(_) => UcbValue::Select,
      ucb::UcbSuccess::Value(_, v) => UcbValue::Value(v),
    }
  }
}

/// Statistics for a specific game action.
///
/// This type is used for reporting summary statistics for the next decision to
/// make after executing search.
#[derive(Clone, Debug)]
pub struct ActionStatistics<G>
where
  G: Game,
{
  /// The action.
  pub action: G::Action,
  /// The action's expected payoff.
  pub payoff: G::Payoff,
  /// The result of UCB rollout for that action (used for debugging MCTS with
  /// a UCB rollout policy).
  pub ucb: Result<UcbValue, ucb::UcbError>,
}

/// Creates a new search graph suitable for MCTS search through the state space
/// of the game `G`.
pub fn new_search_graph<G>() -> search_graph::Graph<G::State, VertexData, EdgeData<G>>
where
  G: Game,
{
  search_graph::Graph::<G::State, VertexData, EdgeData<G>>::new()
}

/// Epoch-specific settings for a round of MCTS search.
#[derive(Clone, Copy, Debug)]
pub struct SearchSettings {
  /// The number of simulations to run when estimating payout of a new game state.
  pub simulation_count: u32,
  /// The exploration bias term to use for the UCB policy.
  pub explore_bias: f64,
}

/// Search before it has been initialized with a search graph.
pub struct EmptySearch<R, G>
where
  R: Rng,
  G: Game,
{
  rng: R,
  settings: SearchSettings,
  phantom_data: PhantomData<G>,
}

impl<R, G> EmptySearch<R, G>
where
  R: Rng,
  G: Game,
{
  pub fn new(settings: SearchSettings, rng: R) -> Self {
    EmptySearch {
      rng,
      settings,
      phantom_data: PhantomData,
    }
  }

  pub fn initialize<'a>(
    self,
    root_state: G::State,
    graph: &'a search_graph::AppendOnlyGraph<G::State, VertexData, EdgeData<G>>,
  ) -> RolloutPhase<'a, R, G> {
    let root_node = match graph.get_node(&root_state) {
      Some(n) => n,
      None => graph.append_node(root_state.clone(), VertexData::default()),
    };
    RolloutPhase {
      rng: self.rng,
      settings: self.settings,
      graph,
      root_node,
    }
  }
}

/// State of ongoing MCTS search.
pub struct RolloutPhase<'a, R, G>
where
  R: Rng,
  G: Game,
{
  rng: R,
  settings: SearchSettings,
  graph: &'a search_graph::AppendOnlyGraph<G::State, VertexData, EdgeData<G>>,
  root_node: Node<'a, G::State, VertexData, EdgeData<G>>,
}

impl<'a, R, G> RolloutPhase<'a, R, G>
where
  R: Rng,
  G: Game,
{
  pub fn rollout<S: RolloutSelector<G, R>>(
    mut self,
  ) -> Result<ScoringPhase<'a, R, G>, rollout::RolloutError<'a, G, S::Error>> {
    rollout::rollout(
      self.root_node.clone(),
      S::from(&self.settings),
      &mut self.rng,
    )
    .map(|node| ScoringPhase {
      rng: self.rng,
      settings: self.settings,
      graph: self.graph,
      root_node: self.root_node,
      rollout_node: node,
    })
  }
}

pub struct ScoringPhase<'a, R: Rng, G: Game> {
  rng: R,
  settings: SearchSettings,
  graph: &'a search_graph::AppendOnlyGraph<G::State, VertexData, EdgeData<G>>,
  root_node: Node<'a, G::State, VertexData, EdgeData<G>>,
  rollout_node: Node<'a, G::State, VertexData, EdgeData<G>>,
}

impl<'a, R: Rng, G: Game> ScoringPhase<'a, R, G> {
  pub fn score<S: Simulator<G, R>>(mut self) -> Result<BackpropPhase<'a, R, G>, S::Error> {
    let payoff = match G::Payoff::from_state(self.rollout_node.get_label()) {
      Some(p) => p,
      None => {
        S::from(&self.settings).simulate(self.rollout_node.get_label().clone(), &mut self.rng)?
      }
    };
    Ok(BackpropPhase {
      rng: self.rng,
      settings: self.settings,
      graph: self.graph,
      root_node: self.root_node,
      rollout_node: self.rollout_node,
      payoff,
    })
  }
}

pub struct BackpropPhase<'a, R: Rng, G: Game> {
  rng: R,
  settings: SearchSettings,
  graph: &'a search_graph::AppendOnlyGraph<G::State, VertexData, EdgeData<G>>,
  root_node: Node<'a, G::State, VertexData, EdgeData<G>>,
  rollout_node: Node<'a, G::State, VertexData, EdgeData<G>>,
  payoff: G::Payoff,
}

impl<'a, R: Rng, G: Game> BackpropPhase<'a, R, G> {
  pub fn backprop<S: BackpropSelector<'a, G, R>>(mut self) -> ExpandPhase<'a, R, G> {
    backprop::backprop(
      self.rollout_node.clone(),
      &self.payoff,
      &S::from(&self.settings),
      &mut self.rng,
    );
    ExpandPhase {
      rng: self.rng,
      settings: self.settings,
      graph: self.graph,
      root_node: self.root_node,
      rollout_node: self.rollout_node,
    }
  }
}

pub struct ExpandPhase<'a, R: Rng, G: Game> {
  rng: R,
  settings: SearchSettings,
  graph: &'a search_graph::AppendOnlyGraph<G::State, VertexData, EdgeData<G>>,
  root_node: Node<'a, G::State, VertexData, EdgeData<G>>,
  rollout_node: Node<'a, G::State, VertexData, EdgeData<G>>,
}

impl<'a, R: Rng, G: Game> ExpandPhase<'a, R, G> {
  pub fn expand(self) -> RolloutPhase<'a, R, G> {
    if !self.rollout_node.get_data().mark_expanded() {
      self.rollout_node.get_label().for_actions(|action| {
        let mut child_state = self.rollout_node.get_label().clone();
        child_state.do_action(&action);
        let child = match self.graph.get_node(&child_state) {
          Some(n) => n,
          None => self
            .graph
            .append_node(child_state.clone(), Default::default()),
        };
        assert!(self
          .graph
          .append_edge(self.rollout_node.clone(), child, EdgeData::new(action))
          .is_ok());
        true
      });
    }
    RolloutPhase {
      rng: self.rng,
      settings: self.settings,
      graph: self.graph,
      root_node: self.root_node,
    }
  }
}

// impl<R: Rng, G: Game> ScoringPhase<R, G>
// {
//   pub fn get_root_state(&self) -> &G::State {
//     &self.root_state
//   }

//   pub fn
// }

//   pub fn backprop<'s, S: BackpropSelector<'s, G, R>>(&'s self, node: Node<'s, G::State, VertexData, EdgeData<G>>) -> Option<G:State> {
//     backprop::backprop_iter(node, payoff, S::from(&self.settings), &mut self.rng)
//   }

//   /// Runs an entire round of MCTS that searches for the next move to make from
//   /// the root state. Returns statistics for each of the actions that can be
//   /// made from the root state.
//   pub fn step<'a, F>(
//     &mut self,
//     root_state: &G::State,
//     settings: &SearchSettings,
//   ) -> Result<
//     impl Iterator<Item = ActionStatistics<G>> + 'a,
//     SearchError<<X as RolloutSelector<G, R>>::Error, <Z as Simulator<G, R>>::Error>,
//   > {
//     let expanded;
//     let rollout_selector = X::from(*settings);
//     let state_to_expand = {
//       let rollout_target = {
//         // Safe because RolloutPhase can only be constructed when the root state
//         // is in the graph.
//         let root_node = self.graph.get_node(&root_state).unwrap();
//           Some(n) => n,
//           None => return Err(SearchError::NoRootState),
//         };
//         match rollout::rollout(root_node, rollout_selector, &mut self.rng) {
//           Ok(n) => n,
//           Err(RolloutError::Cycle(_)) => panic!("Cycle in rollout"),
//           Err(RolloutError::Selector(e)) => return Err(SearchError::Rollout(e)),
//         }
//       };
//       expanded = rollout_target.get_data().mark_expanded();
//       let rollout_state: G::State = rollout_target.get_label().clone();
//       let payoff = if let Some(payoff) = G::Payoff::from_state(&rollout_state) {
//         // Known payoff for rollout target.
//         payoff
//       } else if expanded {
//         // Rollout found an unvisited edge to a node that was already
//         // expanded. Because this unvisited edge does not actually terminate in
//         // an unexpanded node, we use statistics collected from the node's
//         // children.
//         let statistics = G::Statistics::default();
//         for child in rollout_target.get_child_list().iter() {
//           statistics.increment(&child.get_data().statistics.as_payoff());
//         }
//         statistics.as_payoff()
//       } else {
//         // Rollout found an unexpanded node with no known payoff, so
//         // we simulate a payoff.
//         let simulator = Z::from(*settings);
//         match simulator.simulate(rollout_state.clone(), &mut self.rng) {
//           Ok(p) => p,
//           Err(e) => return Err(SearchError::Simulator(e)),
//         }
//       };
//       let backprop_selector = Y::from(*settings);
//       backprop::backprop(rollout_target, &payoff, &backprop_selector, &mut self.rng);
//       rollout_state
//     };
//     if !expanded {
//       // Expand edges of node discovered by rollout. (This happens after
//       // backprop.)
//       expand::expand::<G>(graph.get_node_mut(&state_to_expand).unwrap());
//     }

//     // Gather statistics of each child of root.
//     let children = graph.get_node(&root_state).unwrap().get_child_list();
//     let child_scores = ucb::child_edge_ucb_scores::<G, R>(
//       &children,
//       self.epoch,
//       settings.explore_bias,
//       &mut self.rng,
//     );
//     Ok(
//       children
//         .iter()
//         .zip(child_scores.into_iter())
//         .map(|(edge, ucb_result)| ActionStatistics {
//           action: edge.get_data().action().clone(),
//           payoff: edge.get_data().statistics.as_payoff(),
//           ucb: ucb_result.map(|s| UcbValue::from(&s)),
//         }),
//     )
//   }

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
// }

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
